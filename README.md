# Limonium

## _An easy way to download Spigot forks_

Limonium uses the API's or Jenkins to download the .jars

## Features

- You can tell it what specific project you want
- You can tell it what specific version you want
- You can tell it what specific build you want
- You can tell it where to save it
- You can tell it what the jar name should be called

## Softwares

1. Paper -> `./limonium paper 1.18.2 latest`
2. Purpur -> `./limonium purpur 1.18.2 latest`
3. Pufferfish -> `./limonium pufferfish 1.18.2 latest`
4. Patina -> `./limonium patina 1.18.2 latest`
5. Mirai -> `./limonium mirai 1.18.2 latest`
6. Spigot -> `./limonium spigot 1.18.2 latest` **NOT RECOMMENDED**\
__If you choose Spigot then it will install BuildTools.jar to ./lmtmp/ then run it__

## Proxies

1. Waterfall -> `./limonium waterfall 1.18 latest`

### Examples

*Usage: &lt;project_id&gt; &lt;version&gt; &lt;build&gt;*

Extra arguments

1. --n `Output jar name Example: --n Paper.jar`
2. --o `The path of where the jar should go Example: --o /mc-servers/hub/`

```
./limonium paper 1.18.2 latest
```

```
./limonium paper 1.18.2 48
```

```
./limonium paper 1.18.2 latest --n Paper.jar
```

```
./limonium paper 1.18.2 latest --n Paper.jar --o /mc-servers/hub/
```

Other Arguments:
`--self-update` this will update Limonium if there's a new Limonium version available.\
*I have a lot of Minecraft Servers so this is helpful for me cause I'm really lazy*
