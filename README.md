anchor idl init -f ./target/idl/lazorkit.json 3CFG1eVGpUVAxMeuFnNw7CbBA1GQ746eQDdMWPoFTAD8
anchor idl init -f ./target/idl/transfer_limit.json HjgdxTNPqpL59KLRVDwQ28cqam2SxBirnNN5SFAFGHZ8
anchor idl init -f ./target/idl/default_rule.json B98ooLRYBP6m6Zsrd3Hnzn4UAejfVZwyDgMFaBNzVR2W
anchor idl upgrade 3CFG1eVGpUVAxMeuFnNw7CbBA1GQ746eQDdMWPoFTAD8 -f ./target/idl/lazorkit.json
anchor idl upgrade HjgdxTNPqpL59KLRVDwQ28cqam2SxBirnNN5SFAFGHZ8 -f ./target/idl/transfer_limit.json
anchor idl upgrade B98ooLRYBP6m6Zsrd3Hnzn4UAejfVZwyDgMFaBNzVR2W -f ./target/idl/default_rule.jsona