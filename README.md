# Limonium

## _An easy way to download Spigot forks_

Limonium uses platform specific download APIs or Jenkins to download the .jars

## Features

- You can tell it what specific project you want
- You can tell it what specific version you want
- You can tell it where to save it

## Softwares

1. [Paper](https://github.com/PaperMC/Paper) -> `./limonium paper 1.19.2`
2. [Purpur](https://github.com/PurpurMC/Purpur) -> `./limonium purpur 1.19.2`
3. [Pufferfish](https://github.com/pufferfish-gg/Pufferfish) -> `./limonium pufferfish 1.19.2`
4. [Petal](https://github.com/Bloom-host/Petal) -> `./limonium petal 1.19.2`
5. [Mirai](https://github.com/etil2jz/Mirai) -> `./limonium mirai 1.19.2` **NOT RECOMMENDED**
6. Spigot -> `./limonium spigot 1.19.2` **NOT RECOMMENDED**\
__If you choose Spigot then it will install BuildTools.jar to ./lmtmp/ then run it__

## Proxies

1. [Waterfall](https://github.com/PaperMC/Waterfall) -> `./limonium waterfall 1.19.2`

## Examples

*Usage: &lt;project_id&gt; &lt;version&gt; &lt;build&gt;*

Extra arguments
1. --o `The path of where the jar should go Example: --o /mc-servers/hub/Paper.jar`

```
./limonium paper 1.19.2
```

```
./limonium paper 1.19.2 --o Paper.jar
```

```
./limonium paper 1.19.2 --o ./mc-servers/hub/Paper.jar
```

Other Arguments:
`--self-update` this will update Limonium if there's a new Limonium version available.\
*I have a lot of Minecraft Servers so this is helpful for me because I'm really lazy*

## Why Rust

Why? I wanted to learn a new language, so I chose Rust. Doing something small like this in a new language can help you get comfortable with a language.
