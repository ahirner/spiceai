# Spice.ai Data Components

This crate implements DataFusion TableProviders for reading, writing and streaming Arrow data to/from various systems.

Each component has up to 3 capabilities:
- **Read**: **Required** Read data from the component, implemented via TableProvider.scan().
- **Write**: **Optional** Write data to the component, implemented via TableProvider.insert_into().
- **Stream**: **Optional** Stream data from the component, implemented via a TableProvider.scan() that returns an ExecutionPlan with `unbounded_output` set to `true`.