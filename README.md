anchor idl init -f ./target/idl/lazorkit.json HJoSAFHenQfaYuMgYZ8ZfhsRsuSZ8WYDSVm788DqvVEw
anchor idl init -f ./target/idl/wallet_management_contract.json 3CZwSHhvGhvwiNs1AWAUjeww3UdnVGB4mLvuB2ZQgRQE
anchor idl upgrade HJoSAFHenQfaYuMgYZ8ZfhsRsuSZ8WYDSVm788DqvVEw -f ./target/idl/lazorkit.json
anchor idl upgrade 3CZwSHhvGhvwiNs1AWAUjeww3UdnVGB4mLvuB2ZQgRQE -f ./target/idl/wallet_management_contract.json