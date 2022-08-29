# Limonium

## _An easy way to download Spigot forks_

Limonium uses platform specific download APIs or Jenkins to download the .jars

## Features

- You can tell it what specific project you want
- You can tell it what specific version you want
- You can tell it what specific build you want
- You can tell it where to save it

## Softwares

1. Paper -> `./limonium paper 1.19.2 latest`
2. Purpur -> `./limonium purpur 1.19.2 latest`
3. Pufferfish -> `./limonium pufferfish 1.19.2 latest`
4. Petal -> `./limonium petal 1.19.2 latest`
5. Mirai -> `./limonium mirai 1.19.2 latest`
6. Spigot -> `./limonium spigot 1.19.2 latest` **NOT RECOMMENDED**\
__If you choose Spigot then it will install BuildTools.jar to ./lmtmp/ then run it__

## Proxies

1. Waterfall -> `./limonium waterfall 1.19.2 latest`

## Examples

*Usage: &lt;project_id&gt; &lt;version&gt; &lt;build&gt;*

Extra arguments
1. --o `The path of where the jar should go Example: --o /mc-servers/hub/Paper.jar`

```
./limonium paper 1.19.2 latest
```

```
./limonium paper 1.19.2 48
```

```
./limonium paper 1.19.2 latest --o Paper.jar
```

```
./limonium paper 1.19.2 latest --o ./mc-servers/hub/Paper.jar
```

Other Arguments:
`--self-update` this will update Limonium if there's a new Limonium version available.\
*I have a lot of Minecraft Servers so this is helpful for me cause I'm really lazy*

## Why Rust

Why? I wanted to learn a new language, so I chose Rust. Doing something small like this in a new language can help you get comfortable with a language.
