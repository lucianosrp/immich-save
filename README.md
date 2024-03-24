> ⚠️ This project is still WIP

# Immich-save
Load/Save your albums and favorites assets from/to a convinent text file.

## Reasoning 
Immich already provides a good way to make backups and transision libraries and the database.
However, you may still want to have a file that can recreate albums and "re-fav" your assets easily for when things go wrong (e.g. Database becomes impotabible, broken assets, etc.)

The philosophy should be that, if you always have a digital copy of your files, you should always be able to recreate albums and favorites.

Immich-save provides a simple solution for this. 
It stores all the instructions in a convinient yaml file. 
When you reaload your assets into a new version of immich, or you are simply loading them for the first time, you can "load" the savefile into immich and recreate albums and favs.

Ideally, the long-term goal for immich-save would be to export albums and favorites from other media apps such as Gphotos, Prisma into the same common file format to facilitate transistions and sync across services.

## Running immich-save
### Build from source
For now, the only way to run immich-save is to clone the repository and build the binaries using `cargo build --relase`

```bash
git clone https://github.com/lucianosrp/immich-save.git
cd immich-save
cargo build --release && cp target/release/immich-save ~/.local/bin/. 
```


### Commands 
#### Save
To save your albums and favorites you can run
```bash
immich-save -f save.yaml -s http://192.168.1.10:2283 -k q5RKiVq4H7pWlgTyYe840xuBc save
```

This will save everything into the `save.yaml` file which is specified with the flag `-f` 

The server url is specified with the `-s` flag. E.g `http://192.168.1.10:2283`

The API key is specified with the `-k` flag.

#### Load

You can load a savefile into immich by doing the same command as above and just changing `save` with `load`


```bash
immich-save -f save.yaml -s http://192.168.1.10:2283 -k q5RKiVq4H7pWlgTyYe840xuBc load
```


## TODOs
- [ ] Read server and key from file
- [ ] Complete `load` command
- [ ] Improve `load` performance
- [ ] Refactoring and cleanup 