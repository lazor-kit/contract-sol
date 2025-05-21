anchor idl init -f ./target/idl/lazorkit.json DAcTNgSppWiDvfTWa7PMvPmXHAs5DfBnrqRQme8fXJBb
anchor idl init -f ./target/idl/transfer_limit.json HjgdxTNPqpL59KLRVDwQ28cqam2SxBirnNN5SFAFGHZ8
anchor idl init -f ./target/idl/default_rule.json B98ooLRYBP6m6Zsrd3Hnzn4UAejfVZwyDgMFaBNzVR2W
anchor idl upgrade DAcTNgSppWiDvfTWa7PMvPmXHAs5DfBnrqRQme8fXJBb -f ./target/idl/lazorkit.json
anchor idl upgrade HjgdxTNPqpL59KLRVDwQ28cqam2SxBirnNN5SFAFGHZ8 -f ./target/idl/transfer_limit.json
anchor idl upgrade B98ooLRYBP6m6Zsrd3Hnzn4UAejfVZwyDgMFaBNzVR2W -f ./target/idl/default_rule.json