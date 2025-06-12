anchor idl init -f ./target/idl/lazorkit.json 33tS3mSoyvdmKWxb6bgSL657AqH4Wxsu9R6GnvjtZdEd
anchor idl init -f ./target/idl/transfer_limit.json EXYavpYDn6twyPvsGtvuJkEaGeqbN5TLCnC3Fp3evv85
anchor idl init -f ./target/idl/default_rule.json scdFpnHi1Hu1BbKPwEdhRcdWwu5DohSWxCAg3UeDNKZ
anchor idl upgrade 33tS3mSoyvdmKWxb6bgSL657AqH4Wxsu9R6GnvjtZdEd -f ./target/idl/lazorkit.json
anchor idl upgrade EXYavpYDn6twyPvsGtvuJkEaGeqbN5TLCnC3Fp3evv85 -f ./target/idl/transfer_limit.json
anchor idl upgrade scdFpnHi1Hu1BbKPwEdhRcdWwu5DohSWxCAg3UeDNKZ -f ./target/idl/default_rule.jsona