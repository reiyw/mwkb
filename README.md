# mwkb

```bash
$ cargo run --release --bin download_titles <mediawiki_url> <directory>
$ cargo run --release --bin download_markuped_text <mediawiki_url> <directory>
$ cargo run --release --bin parse_markuped_text <directory>
$ cd <directory>
$ ls
biluo
parsed
raw
titles.csv
```

`parsed` directory contains JSON files storing plain text and entities (e.g., [Iron Ore](https://terraria.gamepedia.com/Iron_Ore) from [Terraria](https://terraria.gamepedia.com/Terraria_Wiki)):

```json
{
  "text": "Iron Ore is an early game ore, which spawns on the surface as well as in the Underground and Cavern biomes. Its primary use is to make Iron Bars, which can be used to make the Iron tier of equipment, as well as Buckets, Chains, and many other items. The equivalent of Iron Ore is Lead Ore, which will sometimes replace Iron in a world.  Iron Ore also has a small chance to appear as a bonus drop from Slimes.\nAs Iron bars are needed to create an Iron Anvil, Iron Ore must usually be obtained before any weapons or armor can be created out of metal bars. This can be avoided by purchasing an Iron Anvil from the Merchant, but this is generally inefficient, as it is not difficult to find 15 Iron Ore early on in the game.\n\n\n\n",
  "entities": [
    {
      "start": 26,
      "end": 29,
      "repr": "ore",
      "target": "ore"
    },
    {
      "start": 77,
      "end": 88,
      "repr": "Underground",
      "target": "Underground"
    },
    {
      "start": 93,
      "end": 99,
      "repr": "Cavern",
      "target": "Cavern"
    },
    {
      "start": 135,
      "end": 144,
      "repr": "Iron Bars",
      "target": "Iron Bar"
    },
    {
      "start": 211,
      "end": 218,
      "repr": "Buckets",
      "target": "Bucket"
    },
    {
      "start": 220,
      "end": 226,
      "repr": "Chains",
      "target": "Chain"
    },
    {
      "start": 280,
      "end": 288,
      "repr": "Lead Ore",
      "target": "Lead Ore"
    },
    {
      "start": 385,
      "end": 395,
      "repr": "bonus drop",
      "target": "Bonus drops"
    },
    {
      "start": 401,
      "end": 407,
      "repr": "Slimes",
      "target": "Slimes"
    },
    {
      "start": 446,
      "end": 456,
      "repr": "Iron Anvil",
      "target": "Iron Anvil"
    },
    {
      "start": 611,
      "end": 619,
      "repr": "Merchant",
      "target": "Merchant"
    }
  ]
}
```