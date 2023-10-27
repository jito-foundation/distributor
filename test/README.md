Tests for merkle-tree, written in [Bankrun](https://github.com/kevinheavey/solders).

To run:
- install python 3.10 + deps (see `test/requirements.txt`) 
- go to `programs/` directory
```shell
anchor build
anchorpy client-gen  target/idl/merkle_distributor.json ./test/client_py --program-id [PROGRAM_ID]
```
- run tests