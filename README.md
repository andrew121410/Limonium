# Limonium

## _An easy way to download Spigot forks_

Limonium uses platform specific download APIs or Jenkins to download the .jars

## Features

- You can tell it what specific project you want
- You can tell it what specific version you want
- You can tell it where to save it

## Softwares

1. [Paper](https://github.com/PaperMC/Paper) -> `./limonium paper 1.19.3`
2. [Purpur](https://github.com/PurpurMC/Purpur) -> `./limonium purpur 1.19.3`
3. [Pufferfish](https://github.com/pufferfish-gg/Pufferfish) -> `./limonium pufferfish 1.19.3`
6. [Spigot](https://hub.spigotmc.org/stash/projects/SPIGOT/repos/spigot/browse) -> `./limonium spigot 1.19.3` (Not recommended)\
__If you choose Spigot then it will install BuildTools.jar to ./lmtmp/ then run it__

## Proxies

1. [Waterfall](https://github.com/PaperMC/Waterfall) -> `./limonium waterfall 1.19`

## Important

Limonium is not affiliated with any of the projects listed. It is just a tool to make it easier to download them.

_Note: When using `-serverjars.com` argument some choices may not work as they may not be added to serverjars.com_

## Examples

*Usage: &lt;project_id&gt; &lt;version&gt;*

Extra arguments
1. --o `The path of where the jar should go Example: --o /mc-servers/hub/Paper.jar`
2. --serverjars.com `When this argument is used it will download the jar from` [ServerJars.com](https://serverjars.com/) `instead`
2. --self-update `Updates limonium if there is a new version available`

```
./limonium paper 1.19.3
```

```
./limonium paper 1.19.3 --o Paper.jar
```

```
./limonium paper 1.19.3 --o ./mc-servers/hub/Paper.jar
```

## Building

`cargo build -r`