# AttoDB

A (WIP) distributed key-value store.

Utilises a simple command-based system, similar to Redis.

Communication is via a custom binary format, described in more detail below.

## Commands (Planned / Implemented)
- [ ] PING
- [x] GET
- [x] SET
- [ ] INCR

## Binary Format

All valid messages start with a single byte which designates the "variant" of the message.

| **variant** | **byte** |
| ----------- | -------- |
| PING        | 0x00     |
| COMMAND     | 0x01     |
| OK          | 0x02     |
| NULL        | 0x03     |
| INT         | 0x04     |
| TEXT        | 0x05     |
| ERR         | 0x06     |

The rest of the message depends on the variant, except PING, OK and NULL, which don't have any additional data.
All integers are encoded in big-endian format.

### COMMAND
- command variant (1 byte)
- arg count (1 byte)

Then, repeatedly (for `arg count`):

- length (2 bytes)
- bytes (`length` bytes)

### INT
- 32 bit signed integer

### TEXT - Contains a string message
- length (2 bytes)
- bytes (`length` bytes)

### ERR - Contains a string representing the error message
- length (2 bytes)
- bytes (`length` bytes)

**Command variants and their byte representations**

| **variant** | **byte** |
| ----------- | -------- |
| GET         | 0x00     |
| SET         | 0x01     |
