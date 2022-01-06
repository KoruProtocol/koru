
# Koru


## About

This is a prototype for Koru, a parametrized monetary system consisting of:
- Mutual credit
- Voting and decision making platform (for economic governance)
- Open credit scoring (Reputation)
- Credit underwriting
- Fractional reserve and exchange service for fiat/crypto

to learn more checkout : https://koru.finance

## Environment Setup

1. Install the holochain dev environment (only nix-shell is required): https://developer.holochain.org/docs/install/
2. Enable Holochain cachix with:

```bash
nix-env -iA cachix -f https://cachix.org/api/v1/install
cachix use holochain-ci
```

3. Clone this repo and `cd` inside of it.
4. Enter the nix shell by running this in the root folder of the repository: 

```bash
nix-shell
npm install
```

This will install all the needed dependencies in your local environment, including `holochain`, `hc` and `npm`.

## Building the DNA

- Build the DNA (assumes you are still in the nix shell for correct rust/cargo versions from step above):

```bash
npm run build:happ
```

## Running the DNA tests

```bash
npm run test
```

## Start

To run the built happ:


```
    npm run start
```

This will spawn an orchestrator (API) and a UI. The network configuration is for *local only*. Production ready configuration with bootstrap server is not available yet.
## Package

To package the web happ:

``` bash
npm run package
```

You'll have the `koru.webhapp` in `workdir`. This is what you should distribute so that the Holochain Launcher can install it.

You will also have its subcomponent `koru.happ` in the same folder`.

## Documentation

We are using this tooling:

- [NPM Workspaces](https://docs.npmjs.com/cli/v7/using-npm/workspaces/): npm v7's built-in monorepo capabilities.
- [hc](https://github.com/holochain/holochain/tree/develop/crates/hc): Holochain CLI to easily manage Holochain development instances.
- [@holochain/tryorama](https://www.npmjs.com/package/@holochain/tryorama): test framework.
- [@holochain/conductor-api](https://www.npmjs.com/package/@holochain/conductor-api): client library to connect to Holochain from the UI.
