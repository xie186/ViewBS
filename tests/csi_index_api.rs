use std::ffi::OsString;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use noodles_bgzf as bgzf;
use noodles_core::Position;
use noodles_csi as csi;
use tempfile::tempdir;
use viewbs::meth::Context;
use viewbs::{meth_one_region, DepthFilter, MethOneRegionArgs, PlotOptions, SampleSpec};

fn append_extension(path: &Path, extension: &str) -> PathBuf {
    let mut value = OsString::from(path.as_os_str());
    value.push(".");
    value.push(extension);
    PathBuf::from(value)
}

fn write_csi_indexed_methyl_file(path: &Path) -> std::io::Result<()> {
    use csi::binning_index::index::{
        header::{format::CoordinateSystem, Format},
        reference_sequence::{bin::Chunk, index::BinnedIndex},
        Header,
    };

    let records = [
        ("chr2", 1006_u64, 2_u64, 31_u64, "CG", "CGA"),
        ("chr2", 1007, 6, 36, "CHG", "CAG"),
        ("chr2", 1010, 8, 34, "CG", "CGA"),
        ("chr2", 1012, 3, 30, "CG", "CGA"),
    ];

    let reference_sequence_names = ["chr2".into()].into_iter().collect();
    let header = Header::builder()
        .set_format(Format::Generic(CoordinateSystem::Gff))
        .set_reference_sequence_name_index(0)
        .set_start_position_index(1)
        .set_end_position_index(None)
        .set_reference_sequence_names(reference_sequence_names)
        .build();
    let mut indexer = csi::binning_index::Indexer::<BinnedIndex>::default().set_header(header);
    let mut writer = File::create(path).map(bgzf::io::Writer::new)?;
    let mut start_virtual_position = writer.virtual_position();

    for (chrom, position, methylated, unmethylated, context, trinuc) in records {
        writeln!(
            writer,
            "{chrom}\t{position}\t+\t{methylated}\t{unmethylated}\t{context}\t{trinuc}"
        )?;

        let end_virtual_position = writer.virtual_position();
        let position = Position::try_from(position as usize)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidInput, error))?;
        let chunk = Chunk::new(start_virtual_position, end_virtual_position);
        indexer.add_record(Some((0, position, position, true)), chunk)?;
        start_virtual_position = end_virtual_position;
    }

    writer.finish()?;
    let index = indexer.build(1);
    csi::fs::write(append_extension(path, "csi"), &index)
}

#[test]
fn meth_one_region_queries_csi_index_when_tbi_is_absent() {
    let tmp = tempdir().unwrap();
    let input = tmp.path().join("sample.tab.gz");
    write_csi_indexed_methyl_file(&input).unwrap();
    assert!(!append_extension(&input, "tbi").exists());
    assert!(append_extension(&input, "csi").exists());

    let output = meth_one_region(
        MethOneRegionArgs {
            samples: vec![SampleSpec {
                path: input,
                name: "WT".to_string(),
                region_path: None,
            }],
            outdir: tmp.path().to_path_buf(),
            prefix: "one".to_string(),
            region: "chr2:1006-1010".to_string(),
            flank: 0,
            contexts: vec![Context::Cg],
            depth: DepthFilter { min: 1, max: 100 },
            plot: PlotOptions {
                enabled: false,
                ..PlotOptions::default()
            },
        },
        None,
        None,
    )
    .unwrap();

    assert_eq!(output.summary.records_read, 3);
    assert_eq!(output.summary.records_used, 2);
}
