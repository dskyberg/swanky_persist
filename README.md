# Swanky Persist

Simple Rust based cached persistance model using Mongodb and Redis.

The primary goal of this crate is to make caching and persisting data easy. The crate exposes two
traits. [Cacheable](./src/cacheable.rs) and [Persistable](./src/persistable.rs). Once these traits are impleemented for your data model, you and persist away!

Brought to you by the Swankymutt himself.

## Configuration

The configuration, managed by [DataServicesConfig](./src/data_services_config.rs), is designed to be thread safe.
This isn't needed at all at this point.  But it allows a single instance of the config to be shared across the DB
and Cache instances.  Given the primary target of this crate is to manage data services for web servers, it
makes sense to make the data thread safe to start with.

Create a .env file and add the following.  Note: there are no defaults.  You must set these.

```env
SWANKY_DB_DATABASE=demo
SWANKY_DB_APP_NAME=demo
SWANKY_DB_URI=mongodb://127.0.0.1:27017
SWANKY_CACHE_URI=redis://127.0.0.1
```
## Running

Due to the licensing restrictions for Docker for Mac, I am using [Colima](https://github.com/abiosoft/colima).

```bash
colima start
```
Note: the 8.1 version of the Homebrew instance is broken (see [this issue](https://github.com/actions/runner-images/issues/8104)).  Uninstall all versions of qemu and use 8.0.3

```bash
brew uninstall qemu
curl -OSL https://raw.githubusercontent.com/Homebrew/homebrew-core/dc0669eca9479e9eeb495397ba3a7480aaa45c2e/Formula/qemu.rb
brew install ./qemu.rb
```

Now you can just use standard Docker commands

This crate contains a demo [docker-compose.yaml](./docker-compose.yaml) file for testing purposes.  But obviously, you can use your own.

Assuming you have Docker installed, then just run this to launch the demo services:

```bash
docker-compose up -d
```
