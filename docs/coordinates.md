# Coordinate Compatibility

ViewBS keeps the legacy coordinate behavior by default. This is intentional for
parity with existing command lines and published outputs.

## One-Region Strings

Commands that accept a single region, such as `MethOneRegion`, use:

```text
chr:start-end
```

This format is one-based and closed. For example, `chr2:1006-1010` includes
positions 1006, 1007, 1008, 1009, and 1010.

When a flank is applied to a one-region string, the start coordinate is clipped
at 1 and the end coordinate is extended by the flank length.

## BED-like Region Files

Region files are BED-like tab-delimited files:

```text
chrom  start  end  name  strand
```

Only the first three columns are required. The fourth column is treated as an
optional name. The fifth column is treated as an optional strand for commands
that need strand-aware binning, such as `MethOverRegion`.

For legacy compatibility, ViewBS currently treats these BED-like `start` and
`end` columns as one-based closed coordinates, not zero-based half-open BED
coordinates. For example:

```text
chr2    1006    1010    gene1    -
```

represents five positions, 1006 through 1010 inclusive.

## Indexed Queries

Indexed methylation files are queried with Tabix/CSI region strings derived from
the command input. The generated query strings are also one-based closed
intervals, such as:

```text
chr2:1006-1010
```

## Future Coordinate Modes

Cleaner or more standard coordinate modes can be added later, but they must be
behind explicit flags. The default mode should not silently change because that
would alter scientific outputs for existing ViewBS workflows.
