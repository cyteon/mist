# Configuration

The configuration resides in config.toml, and can be easily modified by the user. You can write comments in the config by prefixing them with #, like `# comment here`.

### Host
The IP the server will bind to \
Default value: `0.0.0.0`

### Port
The port the server will bind to \
Default value: `25565`

### Motd
The server's motd, displayed in the minecraft server list \
Default value: `An mist server`

### Max Players
The max amount of players that can concurrently play on the server \
Default value: `10`

### Online Mode
If set to false, players will not be authenticated \
Default value: `true`

### View Distance
The server's max view distance \
Default value: `8`

### Simulation Distance
The server's max simulation distance (not implemented yet) \
Default value: `8`

### World Name
The name of the current world, used to determine what folder to save the world to \
Default value: `world`

### World Seed
The seed used for world generation (not implemented yet) \
Default value: `<random unsigned 64 bit integer>`
