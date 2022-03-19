# Changelog

## v0.2.3
- Fix argument parsing

## v0.2.2
- Anonymization: output each format into its own directory
- Support saving/loading anonymizer state (mapping of players <=> ID numbers)

## v0.2.1
- Fix a bug when parsing ELOs.

## v0.2.0
- Ties are now properly anonymized.
- ELO is now included in anonymized logs, but is rounded to the nearest 50 ELO.
- A `--no-log` option has been added to the anonymization subcommand to remove the `log` and `inputLog` attributes to save space.