# Topologies

This directory contains named peer graphs for simulator runs.

Recommended first topologies:

- `two-peer-sync`
- `three-peer-consistency`
- `fault-injection`

Each topology should define:

- participating peer IDs
- peer roles
- bootstrap relationships
- which fixture set to load
- expected result summary

## Schema

- `topology.schema.json` is the formal contract for topology files in this directory.
- New topology files should validate against this schema.
- The schema keeps peer definitions explicit without forcing one runtime implementation.
