use std::fs::File;
use std::io::prelude::*;
use std::panic;

use failure::Error;
use parse_wiki_text::{Configuration, Node};

use data::{Data, parse_pageid};

#[derive(Serialize, Debug)]
pub struct Doc {
    text: String,
    entities: Vec<Entity>,
}

#[derive(Serialize, Debug)]
struct Entity {
    start: u32,
    end: u32,
    repr: String,
    target: String,
}

fn collect_text<'a>(nodes: &Vec<Node<'a>>) -> Vec<&'a str> {
    nodes
        .iter()
        .filter_map(|node| match node {
            Node::Text { value, .. } => Some(*value),
            _ => None,
        }).collect()
}

fn is_heading_should_break(nodes: &Vec<Node>) -> bool {
    let texts = collect_text(nodes);
    match texts.first() {
        Some(t) => *t == "References" || *t == "History" || *t == "Video" || *t == "Gallery",
        None => false,
    }
}

impl Doc {
    pub fn parse(text: &str) -> Result<Doc, Error> {
        let res = panic::catch_unwind(|| {
            let mut count = 0;
            let mut doc_text = String::new();
            let mut entities = Vec::new();
            let result = Configuration::default().parse(text);
            for node in result.nodes {
                match node {
                    Node::Text { value, .. } => {
                        count += value.len();
                        doc_text.push_str(value);
                    }
                    Node::Link { target, text, .. } => {
                        let start = count as u32;
                        let mut repr = String::new();
                        let inner_text = collect_text(&text);
                        for val in inner_text {
                            count += val.len();
                            doc_text.push_str(val);
                            repr.push_str(val);
                        }
                        let end = count as u32;
                        entities.push(Entity {
                            start,
                            end,
                            repr,
                            target: target.to_string(),
                        })
                    }
                    Node::Heading { ref nodes, .. } if is_heading_should_break(nodes) => break,
                    Node::ParagraphBreak { .. } | Node::Heading { .. } => {
                        count += 1;
                        doc_text.push_str("\n");
                    }
                    _ => (),
                }
            }
            Doc {
                text: doc_text,
                entities,
            }
        });
        match res {
            Ok(doc) => Ok(doc),
            Err(_) => Err(format_err!("parse failed"))
        }
    }
}

pub fn parse_all_markuped_text(data_dir: &str) -> Result<(), Error> {
    let data = Data::new(data_dir);
    for entry in data.markuped_text_files()? {
        match entry {
            Ok(path) => {
                let pageid = parse_pageid(&path);
                let mut file = File::open(path)?;
                let mut text = String::new();
                file.read_to_string(&mut text)?;
                let res = Doc::parse(&text[..]);
                match res {
                    Ok(doc) => data.save_parsed_text(pageid, &doc)?,
                    Err(_) => eprintln!("page {} parse failed", pageid),
                }
            }
            Err(e) => {
                eprintln!("{:?}", e);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ore() {
        let text = r#"[[File:Ore Layout.png|thumb|300px||All of the current ores.]]
'''Ores''' are rare stone blocks used to obtain resources.

== Usage ==
[[File:Extracted Ores.png|200px|thumb|Ores isolated down to bedrock]]
Ores are primarily collected for [[crafting]] purposes, such as for [[tools]] and [[armor]]. Redstone is obtained from [[redstone ore]], which can be used to create [[redstone circuits]]. Ore can also be combined to create a block of the material's type.
[[File:PercentOfOreByHeight.png|200px|thumb|This graph shows the actual amount of ore found in a relatively small but untapped world.]]
Obtaining resources from ores is as simple as mining them (this is not the case for iron and gold ores, which must be smelted). Coal, diamond, emerald, and nether quartz ores drop 1 unit corresponding raw materials. Redstone ore drops 4-5 redstone dust, and lapis lazuli ore drops 4-8 [[lapis lazuli]]. Note this can be increased greatly with [[fortune]].  All ore blocks except iron and gold require a [[Silk Touch]] pickaxe to drop themselves.

=== As a smelting ingredient ===

{{smelting
|ingredients=Any raw ore + Any [[fuel]]
|Coal Ore; Iron Ore; Lapis Lazuli Ore; Gold Ore; Redstone Ore; Diamond Ore; Emerald Ore; Nether Quartz Ore
|Coal; Iron Ingot; Lapis Lazuli; Gold Ingot; Redstone Dust; Diamond; Emerald; Nether Quartz
}}

=== Further uses ===

All mineral blocks except quartz are crafted by putting 9 of the raw material items in a square. Quartz blocks are irreversibly crafted with 4 nether quartz and can be crafted in the inventory.

{{Crafting
|ingredients=Any processed ore
 |A1=Coal; Iron Ingot; Lapis Lazuli; Gold Ingot; Redstone Dust; Diamond; Emerald;
 |B1=Coal; Iron Ingot; Lapis Lazuli; Gold Ingot; Redstone Dust; Diamond; Emerald;
 |C1=Coal; Iron Ingot; Lapis Lazuli; Gold Ingot; Redstone Dust; Diamond; Emerald;
  |A2=Coal; Iron Ingot; Lapis Lazuli; Gold Ingot; Redstone Dust; Diamond; Emerald; Nether Quartz
  |B2=Coal; Iron Ingot; Lapis Lazuli; Gold Ingot; Redstone Dust; Diamond; Emerald; Nether Quartz
  |C2=Coal; Iron Ingot; Lapis Lazuli; Gold Ingot; Redstone Dust; Diamond; Emerald;
   |A3=Coal; Iron Ingot; Lapis Lazuli; Gold Ingot; Redstone Dust; Diamond; Emerald; Nether Quartz
   |B3=Coal; Iron Ingot; Lapis Lazuli; Gold Ingot; Redstone Dust; Diamond; Emerald; Nether Quartz
   |C3=Coal; Iron Ingot; Lapis Lazuli; Gold Ingot; Redstone Dust; Diamond; Emerald;
  |Output=Block of Coal; Block of Iron; Lapis Lazuli Block; Block of Gold; Block of Redstone; Block of Diamond; Block of Emerald; Block of Quartz
  |ignoreusage=1
}}

All raw materials except quartz can be crafted from their mineral blocks.

{{Crafting
  |ingredients=Most mineral blocks
  |Block of Coal; Block of Iron; Lapis Lazuli Block; Block of Gold; Block of Redstone; Block of Diamond; Block of Emerald
  |Output=Coal, 9; Iron Ingot, 9; Lapis Lazuli, 9; Gold Ingot, 9; Redstone Dust, 9; Diamond, 9; Emerald, 9
  |ignoreusage=1
}}

== Availability ==



The following is the distribution of ores according to the [[altitude]] (layers are number of blocks above the lowest layer of [[bedrock]]). The highest layers that gold, redstone, diamond, and lapis lazuli can be found on are two layers lower.<ref>http://i.imgur.com/djSvZ.png</ref> The graph on the right shows the ore distribution.

{| class="wikitable sortable" data-description="Ores by underground layer"
|-
! Ore type
! Most found on...
! Commonly found up to...
! Rare on layers...
! None at or above...
! Lowest tier of pickaxe needed for drop
! Found in...
|-
| {{BlockLink|id=coal-ore|Coal Ore|Coal}}<ref group="note">This refers to coal ore blocks spawned in ore veins and not as parts of [[fossils]] or [[mansion]] forges.</ref>
| Layers 5-52
| Layer 128
| Layer 129-131
| Layer 132
| {{ItemLink|Wooden Pickaxe|Wooden}}
| {{EnvLink|Overworld}}
|-
| {{BlockLink|id=iron-ore|Iron Ore|Iron}}
| Layers 5-54
| Layer 64
| Layer 65-67
| Layer 68
| {{ItemLink|Stone Pickaxe|Stone}}
| {{EnvLink|Overworld}}
|-
| {{BlockLink|Lapis Lazuli Ore|Lapis Lazuli}}
| Layers 14-16<ref group="note">Unlike other ores, lapis lazuli's frequency peaks around layer 15, and slowly tapers off above and below.</ref>
| Layer 23
| Layer 31-33
| Layer 34
| {{ItemLink|Stone Pickaxe|Stone}}
| {{EnvLink|Overworld}}
|-
| {{BlockLink|id=gold-ore|Gold Ore|Gold}}
| Layers 5-29
| Layer 29
| Layer 31-33
| Layer 34
| {{ItemLink|Iron Pickaxe|Iron}}
| {{EnvLink|Overworld}}
|-
| {{BlockLink|id=gold-ore|Gold Ore|Gold - In Badlands}}
| Layers 32-63
| Layer 76
| Layer 77-79
| Layer 80
| {{ItemLink|Iron Pickaxe|Iron}}
| {{EnvLink|Overworld}}, in [[Badlands]] biomes only
|-
| {{BlockLink|id=diamond-ore|Diamond Ore|Diamond}}
| Layers 5-12
| Layer 12
| Layer 13-15
| Layer 16
| {{ItemLink|Iron Pickaxe|Iron}}
| {{EnvLink|Overworld}}
|-
| {{BlockLink|id=redstone-ore|Redstone Ore|Redstone}}
| Layers 5-12
| Layer 12
| Layer 13-15
| Layer 16
| {{ItemLink|Iron Pickaxe|Iron}}
| {{EnvLink|Overworld}}
|-
| {{BlockLink|id=emerald-ore|Emerald Ore|Emerald}}<ref group="note">Instead of multi-block veins, emerald ore is placed in scattered blocks.</ref>
| Layers 5-29
| Layer 29
| Layer 30-32
| Layer 33
|{{ItemLink|Iron Pickaxe|Iron}}
| {{EnvLink|Overworld}}, in [[Mountains]] biomes only
|-
| {{BlockLink|Nether Quartz Ore|Nether Quartz}}<ref group="note">Quartz spawns equally in all parts of the Nether (not counting lava and bedrock.)</ref>
| Layers 15-120
| Layer 120
| Layer 124-125
| Layer 126
| {{ItemLink|Wooden Pickaxe|Wooden}}
| {{EnvLink|The Nether}}
|}
{{Notelist}}
Redstone has the same layer and line-size statistics as Diamond, but is generated 8 times per chunk as opposed to 1.

== Table ==

{| class="wikitable" data-description="Ores, resources and mineral blocks"
|-
! || Coal || Iron || Redstone || Gold || Lapis Lazuli || Diamond || Emerald || Nether Quartz
|-
! Ore block
| [[File:Coal Ore.png|75px]] || [[File:Iron Ore.png|75px]] || [[File:Redstone Ore.png|75px]] || [[File:Gold Ore.png|75px]] || [[File:Lapis Lazuli Ore.png|75px]] || [[File:Diamond Ore.png|75px]] || [[File:Emerald Ore.png|75px]] || [[File:Nether Quartz Ore.png|75px]]
|-
! Refined resource
| [[File:Coal.png|75px]] || [[File:Iron Ingot.png|75px]] || [[File:Redstone Dust.png|75px]] || [[File:Gold Ingot.png|75px]] || [[File:Lapis Lazuli.png|75px]] || [[File:Diamond.png|75px]] || [[File:Emerald.png|75px]] || [[File:Nether Quartz.png|75px]]
|-
! Mineral block
| [[File:Block of Coal.png|75px]] || [[File:Block of Iron.png|75px]] || [[File:Block of Redstone.png|75px]] || [[File:Block of Gold.png|75px]] || [[File:Lapis Lazuli Block.png|75px]] || [[File:Block of Diamond.png|75px]] || [[File:Block of Emerald.png|75px]] || [[File:Block of Quartz.png|75px]]
|}

== Video ==

{{Video note|This video is outdated, as [[emerald ore]], [[nether quartz ore]], and [[redstone block]] were added in [[1.3.1]] and [[1.5]], respectively. Also, the video shows [[glistering melon]] being crafted with one [[gold nugget]].}}

{{/video}}

== History ==

{{History|classic}}
{{History||May 21, 2009|link=http://notch.tumblr.com/post/110762705/my-list-on-tile-types-so-far|[[Notch]] shows interest in adding [[iron ore]].}}
{{History||0.0.14a|[[File:Iron Ore.png|32px]] [[File:Coal Ore.png|32px]] [[File:Gold Ore Revision 1.png|32px]] Added [[gold ore]], [[iron ore]], and [[coal ore]].
|These ores could only be found exposed in amounts ranging from roughly 10 to 30 in a vein.}}
{{History||October 24, 2009|link=https://notch.tumblr.com/post/221308991/the-new-block-types-and-new-graphics-for-the-gold|Texture change for [[gold ore]] teased.}}
{{History||0.27 SURVIVAL TEST|[[File:Gold Ore.png|32px]] Changed the texture of [[gold ore]].
|Mining an ore will now give the corresponding block of it.}}
{{History|indev}}
{{History||0.31|[[File:Diamond Ore.png|32px]] Added [[diamond ore]].}}
{{History|infdev}}
{{History||(March 20, 2010)|Random ore is now placed randomly around the terrain.}}
{{History||(March 27, 2010)|Ores will now replace stone upon a world reload.}}
{{History||(March 30, 2010)|The ore change of replacing stone upon a world reload has been reverted.}}
{{History|alpha}}
{{History||v1.0.1|[[File:Redstone Ore.png|32px]] Added [[redstone ore]].}}
{{History|beta}}
{{History||1.2|[[File:Lapis Lazuli Ore.png|32px]] Added [[lapis lazuli ore]].}}
{{History||1.8|snap=?|The highest layer for some ores has been lowered by 2.}}
{{History|java}}
{{History||1.0.0|snap=Beta 1.9 Prerelease 4|[[Coal Ore|Coal]], [[diamond ore|diamond]], [[redstone ore|redstone]], and [[lapis lazuli ore]] will now also drop as ore blocks with [[Silk Touch]].}}
{{History||1.3.1|snap=12w21a|[[File:Ruby Ore.png|32px]] On May 21, 2012, the week before the release of [[12w21a]], [[Jeb]] released a screenshot of himself testing the [[trading]] system.<ref>{{Tweet|jeb|204619936616808451}}</ref> At this time, what would become [[emerald ore]] was [[ruby ore]].<ref name="en_US 12w21">snapshot 12w21a/b ''lang/en_US.lang'': '''tile.oreRuby.name=Ruby Ore'''</ref> For the release of 12w21a, the block was changed to an emerald ore. The texture for emerald ore remained unchanged, as Jeb forgot to commit the new texture in the snapshot.<ref>{{Tweet|jeb|205641953742819328}}</ref>}}
{{History|||snap=12w21b|[[File:Emerald Ore.png|32px]] Changed texture of [[emerald ore]].}}
{{History|||snap=12w22a|All ores except [[gold ore|gold]] and [[iron ore|iron]] (which give [[experience]] when [[smelt]]ed) will now drop experience points when mined.
|[[Emerald ore]] veins will now always consist of single blocks.}}
{{History||1.5|snap=13w01a|[[File:Nether Quartz Ore.png|32px]] Added [[nether quartz ore]].}}
{{History||1.8|snap=14w29a|Ore generation is now a lot faster.}}
{{History|pocket alpha}}
{{History||0.1.0|[[File:Coal Ore.png|32px]][[File:Iron Ore.png|32px]][[File:Redstone Ore.png|32px]][[File:Lapis Lazuli Ore.png|32px]][[File:Gold Ore.png|32px]][[File:Diamond Ore.png|32px]] Added [[coal ore|coal]], [[iron ore|iron]], [[redstone ore|redstone]], [[gold ore|gold]], [[lapis lazuli ore|lapis]] and [[diamond ore]]s.}}
{{History||0.3.0|[[Coal ore]] will now drop [[coal]].}}
{{History||0.3.2|[[Diamond ore]] will now drop a [[diamond]].
|[[Iron Ore|Iron]] and [[gold ore]]s are now [[smelt]]able into metal [[ingot]]s.}}
{{History||0.4.0|[[Lapis lazuli ore]] will now drop the [[lapis lazuli]] dye.}}
{{History||0.8.0|snap=?|Redstone ore will now ''finally'' drop [[redstone dust]].}}
{{History||0.9.0|snap=?|[[File:Emerald Ore.png|32px]] Added [[emerald ore]].}}
{{History||0.12.1|snap=?|[[File:Nether Quartz Ore.png|32px]] Added [[nether quartz ore]].
|Mining ores will now give [[experience]].}}
{{History|foot}}

== Gallery ==

<gallery>
File:Compact Ore Cave.jpg|A picture of every ore except emerald and nether quartz.
File:Every Overworld Ore.png|All of the current overworld ores including emerald ore and diamond ore.
File:LuckyCave.png|All overworld ores in a natural cave.
File:Ores in a Ravine.png|A vein of coal, and a vein of iron, both in a ravine. Ores like this can be hard to get to, due to the difficulty of climbing the [[ravine]]'s wall.
File:Ore Veins.png|[[Generated structures#Mineral vein|Exposed ore veins.]]
File:Pentavein.png|Five different ore veins.
</gallery>

== References ==

{{reflist}}


{{Blocks}}

[[Category:Ore]]

[[cs:Lůžko]]
[[de:Erz]]
[[es:Menas]]
[[fr:Minerai]]
[[hu:Érc]]
[[it:Minerale]]
[[ja:鉱石]]
[[ko:광석]]
[[pl:Rudy]]
[[pt:Minério]]
[[ru:Руды]]
[[tr:Cevher]]
[[zh:矿石]]
"#;
        let doc = Doc::parse(text)?;
        eprintln!("{:#?}", doc);
        eprintln!("{}", doc.text);
        eprintln!("{:#?}", serde_json::to_string_pretty(&doc));
        assert!(false);
    }

    #[test]
    fn test_parse_armor() {
        let text = r#"{{Item
|image=
Leather Cap.png;
Chainmail Helmet.png;
Iron Helmet.png;
Diamond Helmet.png;
Golden Helmet.png;
Turtle Shell.png
|image2=
Leather Tunic.png;
Chainmail Chestplate.png;
Iron Chestplate.png;
Diamond Chestplate.png;
Golden Chestplate.png;
|image3=
Leather Pants.png;
Chainmail Leggings.png;
Iron Leggings.png;
Diamond Leggings.png;
Golden Leggings.png;
|image4=
Leather Boots.png;
Chainmail Boots.png;
Iron Boots.png;
Diamond Boots.png;
Golden Boots.png;
|invimage=Leather Cap
|invimage2=Chainmail Helmet
|invimage3=Iron Helmet
|invimage4=Diamond Helmet
|invimage5=Golden Helmet
|invimage6=Turtle Shell
|invimage7=----
|invimage8=Leather Tunic
|invimage9=Chainmail Chestplate
|invimage10=Iron Chestplate
|invimage11=Diamond Chestplate
|invimage12=Golden Chestplate
|invimage13=----
|invimage14=Leather Pants
|invimage15=Chainmail Leggings
|invimage16=Iron Leggings
|invimage17=Diamond Leggings
|invimage18=Golden Leggings
|invimage19=----
|invimage20=Leather Boots
|invimage21=Chainmail Boots
|invimage22=Iron Boots
|invimage23=Diamond Boots
|invimage24=Golden Boots
|type= Wearable items
|durability= See [[#Durability|Durability]]
|renewable=Yes
|stackable=No
|multipledata= See [[#Data values|Data values]]
|nameid= See [[#Data values|Data values]]
}}
{{More images|single=1|The diamond chestplate uses a slightly different model when worn in Bedrock Edition and old versions of Java Edition (see the sleeves), please include a render}}
{{about||the armor that can be worn by adult horses|Horse Armor}}
[[File:Armorc.png|400px|thumb|Armor tiers. From left (weakest) to right (strongest): no armor, [[leather]], [[gold]]en, chainmail, [[iron]], and [[diamond]]. (Note that the turtle helmet is not pictured. It is between iron and diamond, having equal armor points and greater durability than iron).]]

'''Armor''' is a category of items that provide [[player]]s and certain [[mob]]s with varying levels of protection from common [[damage]] types, and appear graphically on the wearer. These items include several different tiers of [[helmet]]s, [[chestplate]]s, [[leggings]], and [[boots]], which can each be placed in designated armor slots of a player's [[inventory]] for use.

== Obtaining ==

=== Crafting ===

It takes 24 units of a given material to make a full set of armor. Chainmail armor cannot be crafted in survival mode.

Armor can be [[Item Repair|repaired]] by placing two pieces of the same type (e.g., iron helmets) in a crafting grid. The resulting item will have 5% more durability left than the original items combined, but any enchantments will be lost. Repairing armor with an [[anvil]] will preserve and combine the enchantments. Chainmail armor can be repaired with anvils by combining iron ingots with it.

{{:Crafting/Armor}}

=== Drops ===

Zombies and skeletons that spawn with armor have a small chance of dropping their armor when killed by the player. When killed, the armor they drop can vary from 1 to full durability. Zombies, skeletons, zombie pigmen and wither skeletons upon death will always drop armor that they picked up and equipped.

=== Trading ===

Leather pants and enchanted leather tunics can be bought from leatherworker villagers.

Iron helmets, iron chestplates, enchanted diamond chestplates, chainmail helmets, chainmail chestplates, chainmail leggings and chainmail boots can be bought from armorer villagers.

=== Natural generation ===

; Helmets
* {{LootChestItem|enchanted-leather-cap}}
* {{LootChestItem|iron-helmet}}
* {{LootChestItem|enchanted-iron-helmet}}
* {{LootChestItem|golden-helmet}}
* {{LootChestItem|chainmail-helmet}}
* {{LootChestItem|enchanted-diamond-helmet}}
; Chestplates
* {{LootChestItem|enchanted-leather-tunic}}
* {{LootChestItem|leather-tunic}}
* {{LootChestItem|iron-chestplate}}
* {{LootChestItem|enchanted-iron-chestplate}}
* {{LootChestItem|golden-chestplate}}
* {{LootChestItem|chainmail-chestplate}}
* {{LootChestItem|diamond-chestplate}}
* {{LootChestItem|enchanted-diamond-chestplate}}
; Leggings
* {{LootChestItem|enchanted-leather-pants}}
* {{LootChestItem|iron-leggings}}
* {{LootChestItem|enchanted-iron-leggings}}
* {{LootChestItem|chainmail-leggings}}
* {{LootChestItem|enchanted-diamond-leggings}}
; Boots
* {{LootChestItem|enchanted-leather-boots}}
* {{LootChestItem|iron-boots}}
* {{LootChestItem|enchanted-iron-boots}}
* {{LootChestItem|chainmail-boots}}
* {{LootChestItem|enchanted-diamond-boots}}
; Notes
{{notelist}}

== Usage ==
In order to have any protective effect, armor must be worn by the player. Helmets, chestplates, leggings and boots are equipped by placing them in the {{InvSprite|Empty Helmet Slot|scale=0.5|align=text-top}} head, {{InvSprite|Empty Chestplate Slot|scale=0.5|align=text-top}} chest, {{InvSprite|Empty Leggings Slot|scale=0.5|align=text-top}} legs and {{InvSprite|Empty Boots Slot|scale=0.5|align=text-top}} feet slots of the inventory next to your character, respectively. Armor can also be equipped simply by right clicking when held.

Chestplates provide the most protection per unit of material, followed by leggings. For leather, iron, and diamond armor, boots have equivalent armor points as the helmet, but for chainmail and gold armor, the helmets trump boots. Turtle shells, in addition to providing protection, also gives the player the [[Water Breathing]] status effect.

Duplicate armor pieces are not stackable in inventory slots.

=== Tiers ===
[[File:ArmorPE.png|200px|thumb|An outdated picture of the armor section in the then ''Pocket Edition'']]
Armor tiers include (from weakest/least durable to strongest/most durable):
* {{ItemLink|Leather}}
* {{ItemLink|Gold Ingot|Golden}}
* {{ItemSprite|Chainmail Chestplate}} Chainmail
* {{ItemLink|Iron Ingot|Iron}}
* {{ItemLink|Diamond}}

The {{ItemLink|Turtle Shell}} does not fit into a tier, as it is not part of a full set. However, its defense points match gold, chainmail and iron helmets, while its durability is between iron and diamond helmets.

==== Other ====

[[Pumpkin]]s can be worn as a helmet. This will not provide any protection, and it will partially block the player's view, but it does prevent [[endermen]] from becoming aggressive when players look at them.
[[Mob head]]s can also be worn as a helmet. They cut a player's detection range by 50% for the corresponding mob type. This bonus stacks with potions of invisibility and sneaking.

[[Elytra]] can be worn as a chestplate. Like pumpkins they do not provide any defense, but they do allow the player to glide through the air.

=== Dyeing leather armor ===
{{main|Dye#Dyeing armor}}

Leather armor can be dyed, and colors can be mixed.

=== Smelting ingredient ===

{{Smelting|showname=1|head=1|{Any iron armor};{Any chainmail armor}|Iron Nugget}}
{{Smelting|showname=1|foot=1|Any golden armor|Gold Nugget}}

== Mechanics ==

Whenever a piece of armor absorbs damage for the player, the armor itself is damaged, reducing its [[durability]]. After taking enough damage, the armor piece is completely destroyed.

Note that if the damage is absorbed not by the armor itself but by a protection enchantment of the armor, the armor is not damaged. Enchantments can also reduce damage that armor normally does not reduce.

=== Damage types ===

The following types of damage are reduced by armor and, consequently, damage the armor itself:
* Direct attacks from [[mobs]] and [[player]]s
** This includes the [[Strength]] effect and the [[Sharpness]] enchantment.
* Getting hit with an [[arrow]]
** This includes extra damage from enchantments.
* Getting hit with a [[fireball]] from a [[ghast]] or [[blaze]], a [[fire charge]], or [[Ender Dragon#Ender charge|ender acid]]
* Touching [[fire]], [[lava]], [[magma blocks]], or [[cacti]]
* [[Explosion]]s
* Getting struck by lightning
* Getting hit with a falling [[anvil]]
* Getting hit by chicken [[egg]]s
* Getting hit with a [[fishing rod]] lure

The following types of damage are '''not''' reduced by armor and have no effect on the armor itself:
* Ongoing damage from being on [[fire]]
* [[Suffocation|Suffocating]] inside a block
* [[Drowning]] in water (partially for turtle shells)
* [[Food|Starvation]]
* Falling (including [[ender pearl]]s)
* Falling to the [[void]]
* [[Status effect]]s
* Instant damage from a [[potion of Harming]]
* {{cmd|kill}}
* Standing next to where lightning strikes.
* Getting hit by snowballs.

=== Defense points ===

{{fakeImage|{{armorbar|20}}|The armor bar as shown in game}}

Armor defense points are controlled by an [[attribute]], <code>generic.armor</code>. The player's current protection level is represented visually by the armor bar. The armor meter is affected by the particular pieces that are worn as well as the tier of the armor. The following table shows the amount of defense points added by default by each individual piece of armor, as well as the total points added by a full set of armor for each material.

{| class="wikitable" data-description="Defense points"
|-
!scope="col" | Material
!scope="col" | Full set
!scope="col" | Helmet
!scope="col" | Chestplate
!scope="col" | Leggings
!scope="col" | Boots
|-
!scope="row" | Leather
| {{armor|7}}
| {{armor|1}}
| {{armor|3}}
| {{armor|2}}
| {{armor|1}}
|-
!scope="row" | Golden
| {{armor|11}}
| {{armor|2}}
| {{armor|5}}
| {{armor|3}}
| {{armor|1}}
|-
!scope="row" | Chainmail
| {{armor|12}}
| {{armor|2}}
| {{armor|5}}
| {{armor|4}}
| {{armor|1}}
|-
!scope="row" | Iron
| {{armor|15}}
| {{armor|2}}
| {{armor|6}}
| {{armor|5}}
| {{armor|2}}
|-
!scope="row" | Diamond
| {{armor|20}}
| {{armor|3}}
| {{armor|8}}
| {{armor|6}}
| {{armor|3}}
|-
!scope="row" | Turtle Shell
| {{armor|2}}
| {{armor|2}}
| N/A
| N/A
| N/A
|}

Different combinations of armor provide different levels of defense.

=== Armor toughness ===
Armor can further protect the player through a second [[attribute]], <code>generic.armorToughness</code>. By default, only diamond armor provides toughness, with each piece granting +2 toughness.

=== Damage protection ===
Damage taken depends on the number of defense points, the toughness of the armor worn, and the strength of the attack.

The damage formula is <code>damage = damage * ( 1 - min( 20, max( defensePoints / 5, defensePoints - damage / ( toughness / 4 + 2 ) ) ) / 25 )</code>.

Broken down, this means that each armor point gives 4% maximum damage reduction against an incoming attack. Without toughness, this max damage reduction is lessened by 2% for each hit point of the incoming attack. 1 piece of diamond armor, which grants +2 toughness to the player, decreases the defense reduction value for each hit point to 1.6%, 2 diamond pieces decreases it to {{frac|4|3}}% (about 1.3333%), 3 decreases it to {{frac|8|7}}% (about 1.1428%) and 4 decreases it to 1%. The exact formula for the defense reduction in percent is <code>defenseReductionInPercent =
damage * 2 / ( ( toughness / 8 ) + 1 )</code>.

Simply put, as toughness increases, the amount of defense reduction done by high-damaging attacks is diminished, and as toughness approaches a very high value (through commands), the defense reduction caused by high-damaging attacks becomes negligible. The final damage reduction value of the armor is capped at a minimum of 0.8% damage reduction per armor point, and to a maximum of 80% total. If armor is cheated in so that the min cap is larger than the max cap, the min cap will be ignored.

In tabular form (with a toughness of 0), damages are:

<div style="overflow: auto-x">
{{#invoke:Armor info|armorDamageTable}}
</div>

Note that these damage values will be lower if a player is wearing pieces of diamond armor or has toughness added to their armor through commands. Armor values of 16 and above are impossible to obtain without at least one piece of diamond armor, without using cheats.

====Bedrock Edition====
Damage taken only depends on the number of defense points.

Each armor point gives 4% damage reduction, for example, a player wearing a complete set of leather, always gives 28% of damage protection (while in Java it protects 5.6-14%).

=== Enchantments ===
{{See also|Enchanting}}

Armor can be [[enchant]]ed to provide various enchantments. Enchantments can provide more protection or allow armor to protect certain types of damage that armor doesn't normally protect against, such as fall damage or fire. Damage reduction from enchantments do not decrease the armor's durability. Armor enchantments do not appear on the armor bar.

An armor's material determines how enchantable it is. The higher a material's enchantability, the greater the chances of getting multiple and high-level enchantments (see [[enchantment mechanics]] for details).

{| class="wikitable" data-description="Enchantability"
|-
! [[Leather]]
! [[Gold]]en
! Chainmail
! [[Iron]]
! [[Turtle Shell]]
! [[Diamond]]
|-
| 15
| 25
| 12
| 9
| 9
| 10
|}

As with several enchantments, several different levels of protection are possible. The maximum level of a protection enchantment is currently IV (4). Protection enchantments from multiple pieces of armor stack together, up to a calculated maximum.

Each protection enchantment protects against specific types of damage. The amount of damage reduction depends on the '''Enchantment Protection Factor''' (EPF) provided by that enchantment.

{| class="wikitable" style="text-align:center" data-description="Enchantment protection factor"
|-
! Enchantment
! Damage reduced for
! Type Modifier
! EPF<br>Level I
! EPF<br>Level II
! EPF<br>Level III
! EPF<br>Level IV
|-
| Protection
| All
| 1
| 1
| 2
| 3
| 4
|-
| Fire Protection
| [[Fire]], [[lava]], and [[blaze]] fireballs
| 2
| 2
| 4
| 6
| 8
|-
| Blast Protection
| [[Explosion]]s
| 2
| 2
| 4
| 6
| 8
|-
| Projectile Protection
| [[bow|Arrow]]s, [[ghast]] and [[blaze]] fireballs
| 2
| 2
| 4
| 6
| 8
|-
| Feather Falling
| Fall damage (including [[ender pearl]]s)
| 3
| 3
| 6
| 9
| 12
|}

When a player or mob wearing armor is subjected to damage, the EPFs of all applicable enchantments are added together, capped at 20, and then damage is reduced as <code>damage = damage * ( 1 - cappedEPF / 25 )</code>, giving a maximum reduction of 80% at EPF 20.

Because of the caps in the calculation, it's possible to max out protection against specific types of damage with only three pieces of armor. For example, two pieces of armor with Blast Protection IV (EPF 8 each) and a single piece with Protection IV (EPF 4) would give a total EPF of 20 versus explosions. Any additional EPF would be wasted against explosions (but might be useful against other types of damage, if applicable).

If the damage is of a type that armor protects against normally, this reduction applies only to the damage that got through the armor. <!-- For example, a full suit of diamond armor reduces damage from attacks by 80%—if each piece of armor also had a Protection IV enchantment (EPF 5 each), the enchantments would further reduce damage by 40% to 80% each time, for a total damage reduction of 88% to 96% (i.e., 80%, plus 40%-80% of the remaining 20%). -->

It is possible using {{cmd|give}} to obtain armor with an enchantment level higher than what is normally obtainable via normal survival. Using this method, a player could give themselves, for example, a full set of diamond armor with a Protection V enchantment on every piece. Following the algorithm above, we find that, since Protection V has an EPF of 5, the armor will exactly reach the maximum EPF of 20 for all types of damage. Any higher Protection enchantments could be used to allow the cap to be reached with only one enchantment, rather than having a full set of enchanted armor, but would be wasted if all pieces shared the same level enchantment.

=== Durability ===

Any hit from a damage source that can be blocked by armor will remove one point of durability from each piece of armor worn for every {{hp|4}} of incoming damage (rounded down, but never below 1). The following chart displays how many hits each piece of armor can endure.

{| class="wikitable" data-description="Durability"
|-
!scope="col" | Material
!scope="col" | Helmet
!scope="col" | Chestplate
!scope="col" | Leggings
!scope="col" | Boots
|-
!scope="row" | [[Leather]] || 56 || 81 || 76 || 66
|-
!scope="row" | [[Gold Ingot|Golden]] || 78 || 113 || 106 || 92
|-
!scope="row" | Chainmail/[[Iron Ingot|Iron]] || 166 || 241 || 226 || 196
|-
!scope="row" | [[Diamond]] || 364 || 529 || 496 || 430
|-
!scope="row" | [[Turtle Shell|Turtle Shell]] || 276 || N/A || N/A || N/A
|}

The chart below shows the durability per unit of material for each piece of armor, compared to that of the boots. Note that the durability per unit does not depend on the tier of the armor.

{| class="wikitable" data-description="Durability / unit"
|-
!scope="col" |
!scope="col" | Helmet
!scope="col" | Chestplate
!scope="col" | Leggings
!scope="col" | Boots
|-
| '''Durability/Unit''' || 68% || 61% || 66% || 100%
|}

This means that for the same number of leather/iron ingots/gold ingots/chainmail/diamond, boots can take 1.5 more damage than leggings. Thus, chestplate and leggings offer more defense points per unit, but have a less durability per unit.

A non-player mob will not lose durability when attacked, by any means other than sunlight.

== Mob armor ==
=== Mobs equipping armor ===
[[File:ZombieIron.png|thumb|150px|Zombie equipped with iron armor]]

Certain mobs can spawn equipped with random armor pieces. Some mobs also spawn with the ability to pick up armor on the ground and equip them. The probability of mobs spawning equipped with armor, whether the armor is enchanted, the level of enchantment of the armor, and how many pieces of armor a mob spawns with depend on the [[difficulty]]; if a mob spawns with armor, the tier of armor (leather, gold, etc.) has a fixed probability:
{| class="wikitable" data-description="Mob armor"
|-
! Armor Type
! Chance
|-
| Leather
| 37.06%
|-
| Gold
| 48.73%
|-
| Chainmail
| 12.90%
|-
| Iron
| 1.27%
|-
| Diamond
| 0.04%
|}
Protection provided by armor and armor enchantments works the same with mobs as it does with players.

The following mobs can spawn with armor:

* {{EntityLink|Zombie}}
** {{EntityLink|id=zombie|Zombie|Baby Zombie}}
** {{EntityLink|id=zombie-villager|Villager Zombie}}
** {{EntityLink|id=zombie-villager|Zombie|Baby Villager Zombie}}
* {{EntityLink|Skeleton}}
* {{EntityLink|Stray}}

The following do not naturally spawn with armor, but will pick up any dropped pieces:

* {{EntityLink|Drowned}}
* {{EntityLink|Zombie Pigman}}
** {{EntityLink|id=zombie-pigman|Zombie Pigman|Baby Zombie Pigman}}
* {{EntityLink|Wither Skeleton}}

These mobs cannot wear armor through Survival mode means, but if equipped with commands, their armor will be visible:

* {{EntityLink|Giant}}

All other mobs can be equipped via commands, although it will not be visible. It is also possible to equip [[villager]]s<!--any other mobs?--> with it via [[dispenser]]s.

Helmets can protect mobs from burning in [[sunlight]], depleting its durability as it absorbs the damage. Eventually, the helmet will lose all its durability and break. Pumpkins and mob heads also protect mobs from burning in sunlight.
Damage caused by any other source will not cause the mob's armor durability to decrease.

=== Horse armor ===
{{main|Horse armor}}

[[Horse armor]] can be equipped on [[horse]]s to protect them from mob and player damage.

=== Armor points ===

Certain mobs naturally have armor points.

{| class="wikitable"
! Mob !! Points
|-
| {{EntityLink|Zombie}}<br>{{EntityLink|Zombie Pigman}}<br>{{EntityLink|Zombie Villager}}<br>{{EntityLink|Husk}}<br>{{EntityLink|Drowned}} || {{armor|2}}
|-
| {{EntityLink|Magma Cube}} (tiny) || {{armor|3}}
|-
| {{EntityLink|Wither}} || {{armor|4}}
|-
| {{EntityLink|Magma Cube}} (small) || {{armor|6}}
|-
| {{EntityLink|Killer Rabbit}} || {{armor|8}}
|-
| {{EntityLink|Magma Cube}} (big) || {{armor|12}}
|-
| {{EntityLink|Shulker}} (when closed) || {{armor|20}}
|}

== Data values ==

{{/ID}}

== Achievements ==

{{load achievements|Iron Man;Tie dye outfit}}

== Advancements ==
{{load advancements|Suit Up;Cover me With Diamonds}}
'''Easter Egg:''' The Cover Me With Diamonds advancement is coded as 'shiny_gear'.

== Video ==

{{/video}}

== History ==
<onlyinclude>
{{More images|3D renders of the "mob armor"; texture files can be found at [[Java Edition removed features#Plate.png]]}}
{{History|classic}}
{{History||June 14, 2009|link=https://notch.tumblr.com/post/123343045/my-vision-for-survival|[[Notch]] discussed how armor would work in [[Survival]] mode: "Two types of swords, two types of armor, two types of helmets. The basic versions require iron. The advanced versions require steel, which you make by combining iron and coal. Carrying swords, armor or helmets take up inventory slots, but otherwise have no penalty and work pretty much as you expect (prevent some damage, or cause more damage)"}}
{{History||August 13, 2009|link={{tumblr|notch|162091556}}|Notch tested with armor models. Only chestplates and helmets were available. They were merely aesthetic at the time and had no effect on gameplay.}}
{{History||0.24_SURVIVAL_TEST|Armor models were tested with zombies and skeletons.}}
{{History||unknown|Armor was later removed from zombies and skeletons.}}
{{History|indev}}
{{History||unknown|{{InvSprite|Studded Helmet}}{{InvSprite|Studded Chestplate}}{{InvSprite|Studded Leggings}}{{InvSprite|Studded Boots}}[[Studded armor]] can be seen in the texture files.}}
{{History||February 9, 2010|link=http://notch.tumblr.com/post/380486636/this-is-going-to-be-a-slow-week|Notch revealed new models for armor, which included leggings and boots.}}
{{History||(February 18, 2010)|{{InvSprite|Leather Cap Revision 1}}{{InvSprite|Leather Tunic Revision 1}}{{InvSprite|Leather Pants Revision 1}}{{InvSprite|Leather Boots Revision 1}} Added the cloth set, given the textures from one of Notch's previous games, ''[[Legend of the Chambered]]''.
|{{InvSprite|Chainmail Helmet}}{{InvSprite|Chainmail Chestplate}}{{InvSprite|Chainmail Leggings}}{{InvSprite|Chainmail Boots}} Added the chain set.
|{{InvSprite|Iron Helmet}}{{InvSprite|Iron Chestplate}}{{InvSprite|Iron Leggings}}{{InvSprite|Iron Boots}} Added the iron set.
|{{InvSprite|Golden Helmet}}{{InvSprite|Golden Chestplate}}{{InvSprite|Golden Leggings}}{{InvSprite|Golden Boots}} Added the gold set.
|{{InvSprite|Diamond Helmet}}{{InvSprite|Diamond Chestplate}}{{InvSprite|Diamond Leggings}}{{InvSprite|Diamond Boots}} Added the diamond set.
|Armor can be crafted and worn.|Armor now functions. All helmets give {{Armor|3}}, all chestplates give {{Armor|8}}, all leggings give {{Armor|6}}, and all boots give {{Armor|3}}. Armors have limited durability, with lower tier armors less durable than higher tier armors.}}
{{History|alpha}}
{{History||v1.0.8|Renamed wool armor to leather armor.|Leather armor is now crafted with leather instead of wool.}}
{{History|java}}
{{History||1.0.0|snap=Beta 1.9 Prerelease|Each tier now provide different amounts of protection.}}
{{History|||snap=October 3, 2011|slink={{tweet|notch|120859830339637249}}|The first images of a player wearing enchanted armor are revealed.}}
{{History|||snap=Beta 1.9 Prerelease 3|Iron armor can now be found in the new [[stronghold]] altar chests.}}
{{History|||snap=Beta 1.9 Prerelease 4|Armor can be enchanted.}}
{{History||1.1|snap=12w01a|Iron armor can be found in the new blacksmith chests in [[village]]s.}}
{{History||1.2.1|snap=12w06a|Zombies drop iron helmets on rare occasions, and zombie pigmen drop golden helmets.}}
{{History||1.3.1|snap=12w15a|{{key|Shift}}+clicking can now be used to wear armor.}}
{{History|||snap=12w21a|Chain armor can now be obtained legitimately in survival mode through [[trading]].
|Blacksmith [[villager]]s now sell chain boots for 5–6 [[emerald]]s, chain leggings for 9–10 emeralds, chain chestplates for 11–14 emeralds and chain helmets for 5–6 emeralds.
|They sell diamond boots for 7 emeralds, diamond leggings for 11–13 emeralds, diamond chestplates for 16–18 emeralds and diamond helmets for 7 emeralds.
|They sell iron boots for 4–5 emeralds, iron leggings for 8–9 emeralds, iron chestplates for 10–13 emeralds and iron helmets for 4–5 emeralds.
|Butchers now sell leather boots, caps and pants each for 2–3 emeralds, and leather tunics for 4 emeralds.}}
{{History||1.4.2|snap=12w32a|Mob armor is reintroduced. A partial or full set of any armor is sometimes worn by zombies, skeletons and zombie pigman, with the likelihood increasing with difficulty.}}
{{History|||snap=August 17, 2012|slink={{tweet|Dinnerbone|236445090929844225}}|Jeb and Dinnerbone tweeted pictures of dyeable leather armor.}}
{{History|||snap=12w34a|Leather armor can now be dyed by crafting a leather armor piece with dyes. Dyes can be removed by {{control|use|text=using}} dyed leather armor on a [[cauldron]] with water.|{{InvSprite|Leather Cap Revision 2}}{{InvSprite|Leather Tunic Revision 2}}{{InvSprite|Leather Pants Revision 2}}{{InvSprite|Leather Boots Revision 2}} Default leather armor texture is slightly darker.}}
{{History|||snap=12w34b|Leather and diamond armor models were altered. Leather tunics have buttons and longer sleeves, and leather caps no longer have a central, narrow protrusion. Diamond chestplates have notches under the shoulders.}}
{{History|||snap=12w36a|Dyed leather armor are more saturated and have a slight tint of tan in respect to the default armor color.|Wither skeletons can spawn with random armor.}}
{{History|||snap=12w37a|{{InvSprite|Leather Cap}}{{InvSprite|Leather Tunic}}{{InvSprite|Leather Pants}}{{InvSprite|Leather Boots}}Leather armor now has non-dyed parts. This was implemented so that players can distinguish between other types of armor and similarly colored leather armor.}}
{{History||1.4.6|snap=12w50a|The [[Thorns]] enchantment can be enchanted on chestplates.}}
{{History||1.5|snap=13w04a|Armor in your hand can be equipped by right-clicking.
|Dispensers can equip nearby players with armor.}}
{{History||1.6.1|snap=13w18a|Golden chestplates are now found in the new chests in [[nether fortress]]es.}}
{{History||1.7.2|snap=13w36a|Leather boots can be obtained as one of the "junk" items by fishing.}}
{{History||1.8|snap=14w02a|Trades changed: armorer [[villager]]s now sell chain boots for 5–7 [[emerald]]s, chain leggings for 9–11 emeralds, chain chestplates for 11–15 emeralds and chain helmets for 5–7 emeralds.
|They sell enchanted diamond chestplates for 16–19 emeralds, and no longer sell other diamond armor.
|They sell iron chestplates for 10–14 emeralds and iron helmets for 4–6 emeralds, and no longer sell other iron armor.
|Leatherworkers now sell leather pants for 2–4 emeralds and enchanted leather tunics for 7–12 emeralds, and no longer sell other leather armor.}}
{{History|||snap=14w05a|Armor no longer turns red when mobs/players are hurt.}}
{{History|||snap=14w06a|Armor is now visible on [[giant]]s.}}
{{History|||snap=14w25a|Chain armor [[Java Edition removed features#Chain armor|cannot be crafted anymore]] due to the item form of [[fire]] being [[Java Edition removed features#Obtainable until 1.8|removed]].}}
{{History||1.9|snap=15w31a|Enchanted iron and diamond armor can now be found in [[end city]] ship chests.
|Mobs now wear armor from the bottom to the top, rather than from the top to the bottom. This means that a mob with three armor pieces, for example, will spawn with all armor except a helmet.}}
{{History|||snap=15w34b|Armor durability now affects armor value.}}
{{History|||snap=15w36a|Armor and armor enchantment calculations changed. For the original values, see [[Armor/Before 1.9|here]].}}
{{History|||snap=15w36d|Armor durability affecting value removed.|Armor now has an attribute controlling the defense points.}}
{{History|||snap=15w43a|Decreased average yield of gold chestplates in [[nether fortress]] chests.}}
{{History|||snap=15w50a|Added <code>equip</code> sounds for all types of armor.}}
{{History|||snap=16w02a|Armor and armor enchantment calculations changed again.}}
{{History|||snap=16w05a|Armor calculations changed again.}}
{{History||1.11|snap=16w39a|Diamond and chainmail chestplates are now found in the new [[woodland mansion]] chests.}}
{{History||1.11.1|snap=16w50a|Golden, chain and iron armor now smelt down into one of their respective nuggets.}}
{{History||1.13|snap=17w47a|Prior to [[1.13/Flattening|''The Flattening'']], these items' numeral IDs were 298 through 317.}}
{{History|||snap=18w07a|{{InvSprite|Turtle Shell}} Added turtle shells.}}
{{History|||snap=18w09a|Leather tunics and golden helmets now have a chance of generating in [[underwater ruins]].}}
{{History|||snap=18w10a|Leather tunics can now generate in [[buried treasure]] chests.}}
{{History|||snap=18w11a|Enchanted leather caps, tunics, pants, and boots can generate in the chests of [[shipwreck]]s.}}
{{History|||snap=18w20a|Chain armor pieces renamed to "chainmail".}}
{{History|pocket alpha}}
{{History||0.6.0|{{InvSprite|Leather Cap Revision 1}}{{InvSprite|Leather Tunic Revision 1}}{{InvSprite|Leather Pants Revision 1}}{{InvSprite|Leather Boots Revision 1}} Added the leather set.
|{{InvSprite|Chainmail Helmet}}{{InvSprite|Chainmail Chestplate}}{{InvSprite|Chainmail Leggings}}{{InvSprite|Chainmail Boots}} Added the chain set.
|{{InvSprite|Iron Helmet}}{{InvSprite|Iron Chestplate}}{{InvSprite|Iron Leggings}}{{InvSprite|Iron Boots}} Added the iron set.
|{{InvSprite|Golden Helmet}}{{InvSprite|Golden Chestplate}}{{InvSprite|Golden Leggings}}{{InvSprite|Golden Boots}} Added the golden set.
|{{InvSprite|Diamond Helmet}}{{InvSprite|Diamond Chestplate}}{{InvSprite|Diamond Leggings}}{{InvSprite|Diamond Boots}} Added the diamond set.}}
{{History||0.8.0|snap=?|{{InvSprite|Leather Cap}}{{InvSprite|Leather Tunic}}{{InvSprite|Leather Pants}}{{InvSprite|Leather Boots}} Changed leather armor sprites to that of the PC version, but its armor model remains that of older versions.|? = }}
{{History||0.9.0|snap=build 1|Iron armor naturally generates in village chests and a stronghold altar chest.}}
{{History||0.11.0|snap=build 11|Armor will now only protect against mob damage.}}
{{History||0.12.1|snap=?|Armor can now be worn by mobs.|Armor no longer turns red when mobs/players are hurt.|Golden Chestplate Can be Found in Nether Fortress Chest.|Chainmail Armor now obtainable in survival via mob that wearing it.|Leather Boots can be obtained from fishing as a junk item.}}
{{History||0.14.0|snap=build 1|Leather armor can now be dyed and the model has been updated.}}
{{History||0.15.0|snap=build 1|Armor can be obtained from [[Stray]] and [[Husk]] that naturally spawn with armor.}}
{{History||0.15.10|[[Cape]]s no longer clip through armor.}}
{{History|pocket}}
{{History||1.0|snap=alpha 0.17.0.1|Enchanted Iron Armor and Enchanted Diamond Armor can be found inside End city chests.}}
{{History||1.0.4|snap=alpha 1.0.4.0|Iron Helmet, Iron Chestplate, Enchated Diamond Chestplate and Chainmail Armor are now sold by Armorer smith Villager via trading.}}
{{History||1.1|snap=alpha 1.1.0.0|Golden, chain and iron armor can now be smelted down into one of their respective nuggets.|Diamond Chestplate and Chainmail Chestplate can be found inside woodland mansion chests.}}
{{History|bedrock}}
{{History||1.4|snap=beta 1.2.14.2|Chainmail helmets, chestplates, leggings, and boots now generate in [[buried treasure]] chests.
|Enchanted Leather Armor now can be found inside [[shipwreck]] supply room chests.}}
{{History|||snap=beta 1.2.20.1|Leather Chestplates and Golden Helmets now can be found inside [[underwater ruins]] chests.}}
{{History||1.5|snap=beta 1.5.0.4|{{InvSprite|Turtle Shell}} Added Turtle Shells}}
{{History|console}}
{{History||xbox=TU1|xbone=CU1|ps=1.0|wiiu=Patch 1|switch=Patch s1|{{InvSprite|Leather Cap Revision 1}}{{InvSprite|Leather Tunic Revision 1}}{{InvSprite|Leather Pants Revision 1}}{{InvSprite|Leather Boots Revision 1}} Added the leather set.
|{{InvSprite|Iron Helmet}}{{InvSprite|Iron Chestplate}}{{InvSprite|Iron Leggings}}{{InvSprite|Iron Boots}} Added the iron set.
|{{InvSprite|Golden Helmet}}{{InvSprite|Golden Chestplate}}{{InvSprite|Golden Leggings}}{{InvSprite|Golden Boots}} Added the golden set.
|{{InvSprite|Diamond Helmet}}{{InvSprite|Diamond Chestplate}}{{InvSprite|Diamond Leggings}}{{InvSprite|Diamond Boots}} Added the diamond set.}}
{{History||xbox=TU5|{{InvSprite|Chainmail Helmet}}{{InvSprite|Chainmail Chestplate}}{{InvSprite|Chainmail Leggings}}{{InvSprite|Chainmail Boots}} Added the chain set.|Added a quick equip for armor to the inventory interface.}}
{{History||xbox=TU12|ps=1.03|{{InvSprite|Leather Cap}}{{InvSprite|Leather Tunic}}{{InvSprite|Leather Pants}}{{InvSprite|Leather Boots}} Changed the texture for leather armor.}}
{{History||xbox=TU14|ps=1.05|Leather armor can be dyed.|[[Item repair]] can repair armors.}}
{{History||xbox=TU25|xbone=CU13|ps=1.16|Armor now has the quick equip functionality.}}
{{History||xbox=TU53|xbone=CU43|ps=1.49|wiiu=Patch 23|switch=Patch s3|Golden, chain and iron armor now smelt down into one of their respective nuggets.}}
{{History|foot}}
</onlyinclude>

== Issues ==

{{issue list}}

== Gallery ==

<gallery>
File:ArmorModel_Aug_13_2009.jpg|First image of armor in Classic.
File:ArmorModel_Feb_9_2010.png|Armor rendering overhaul in Indev.<ref>{{tumblr|notch|380486636|This is going to be a slow week.}}</ref>
File:Zombie armor.png|A zombie wearing a chestplate in Survival Test.
File:Zombie helmet.png|A zombie wearing a helmet in Survival Test.
File:Legend of Chambered.png‎|The armor sprite as it appeared in one of Notch's previous projects, ''[[Legend of the Chambered]]''.
File:InfDev_Cloth_Armor.jpg|Wool chestpiece in Infdev.
File:DinnerboneArmor.png|[[Dinnerbone]]'s first screenshot of dyed armor.
File:ArmorGuide Leather.png|A guide for leather armor.
File:ArmorGuide Gold.png|A guide for golden armor.
File:ArmorGuide IronDiamond.png|A guide for iron and diamond armor.
File:Armor leather.png|Leather armor.
File:Armor_iron.png|Iron Armor.
File:Chainvilleager.png|Trading for chainmail armor with a villager.
File:Leatherarmorcomparison.png|Comparing the old leather armor texture to the new one.
File:Coloredclothes.png|A player equipped with a full set of dyed leather armor.
File:LeatherChain.jpg|The removed texture.
File:Skeleton helmet.png|A [[skeleton]] wearing an iron helmet.
File:Zombie chestplate.png|A [[zombie]] wearing an iron chestplate.
</gallery>

=== Mob armor ===

<gallery>
File:ZombieLeather.png|A [[zombie]] with full leather armor.
File:ZombieGold.png|A [[zombie]] with full gold armor.
File:ZombieChain.png|A [[zombie]] with full chainmail armor.
File:ZombieIron.png|A [[zombie]] with full iron armor.
File:ZombieDiamond.png|A [[zombie]] with full diamond armor.
File:SkeletonRiderGroup.png|[[Skeleton]]s spawned from a skeleton trap [[horse]], with their enchanted iron helmets and bows.
</gallery>

== See also ==
* [[Blocking]]
* [[Shield]]

== References ==

{{Reflist}}

{{Items}}

[[Category:Armor]]

[[cs:Brnění]]
[[de:Rüstung]]
[[es:Armadura]]
[[fr:Armure]]
[[hu:Páncél]]
[[ja:防具]]
[[ko:갑옷]]
[[nl:Harnas]]
[[pl:Zbroja]]
[[pt:Armadura]]
[[ru:Броня]]
[[th:ชุดเกราะ]]
[[zh:盔甲]]
"#;
        let doc = Doc::parse(text)?;
        eprintln!("{:#?}", doc);
        eprintln!("{}", doc.text);
        assert!(false);
    }
}
