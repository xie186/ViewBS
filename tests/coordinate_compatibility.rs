use std::fs;

use tempfile::tempdir;
use viewbs::io::regions::{read_region_records, GenomicRegion};

#[test]
fn one_region_strings_are_one_based_closed_intervals() {
    let region: GenomicRegion = "chr2:1006-1010".parse().unwrap();

    assert_eq!(region.chrom, "chr2");
    assert_eq!(region.start, 1006);
    assert_eq!(region.end, 1010);
    assert_eq!(region.to_tabix_region(), "chr2:1006-1010");
    assert_eq!(region.with_flank(10).to_tabix_region(), "chr2:996-1020");
    assert_eq!(
        "chr2:3-6"
            .parse::<GenomicRegion>()
            .unwrap()
            .with_flank(10)
            .to_tabix_region(),
        "chr2:1-16"
    );
}

#[test]
fn bed_like_region_records_preserve_legacy_closed_coordinates() {
    let tmp = tempdir().unwrap();
    let path = tmp.path().join("regions.bed");
    fs::write(&path, "chr2\t1006\t1010\tgene1\t-\n").unwrap();

    let regions = read_region_records(&path).unwrap();

    assert_eq!(regions.len(), 1);
    assert_eq!(regions[0].chrom, "chr2");
    assert_eq!(regions[0].start, 1006);
    assert_eq!(regions[0].end, 1010);
    assert_eq!(regions[0].len(), 5);
    assert_eq!(regions[0].name.as_deref(), Some("gene1"));
    assert_eq!(regions[0].strand, '-');
    assert_eq!(
        regions[0].to_genomic_region().to_tabix_region(),
        "chr2:1006-1010"
    );
}

#[test]
fn coordinate_documentation_describes_supported_legacy_inputs() {
    let docs = fs::read_to_string("docs/coordinates.md").unwrap();

    assert!(docs.contains("chr:start-end"));
    assert!(docs.contains("one-based"));
    assert!(docs.contains("closed"));
    assert!(docs.contains("BED-like"));
    assert!(docs.contains("legacy"));
}
