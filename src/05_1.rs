use std::{
    collections::{HashMap, HashSet},
    thread::sleep,
    time::{Duration, Instant},
};

use helpers::{
    array_to_rectangle, run, square_at_position, Align, GlowColor, State, TextManager, TextType,
    Timings, SCREEN_HEIGHT, SCREEN_WIDTH,
};
use speedy2d::{
    dimen::Vector2,
    image::{ImageHandle, ImageSmoothingMode},
    Graphics2D,
};

mod helpers;

const EMOJI_SIZE: f32 = 160.0;

struct MyState {
    current_seed_info: Option<(usize, Instant)>,
    seeds: Vec<Seed>,
    maps: HashMap<&'static str, Section>,

    images: Vec<ImageHandle>,
}

impl State for MyState {
    fn on_start(&mut self, graphics: &mut Graphics2D) {
        for i in 0..8 {
            let name = part_index_to_string(i);
            self.images.push(
                graphics
                    .create_image_from_file_path(
                        None,
                        ImageSmoothingMode::NearestNeighbor,
                        &format!("data/{name}.png"),
                    )
                    .unwrap(),
            )
        }
    }
    fn on_draw(
        &mut self,
        _timings: &Timings,
        text_manager: &mut TextManager,
        graphics: &mut Graphics2D,
    ) {
        let seed_duration = self
            .current_seed_info
            .map(|(index, _)| {
                if index <= 2 {
                    8000
                } else if index <= 5 {
                    1000
                } else {
                    50
                }
            })
            .unwrap_or(8000);
        let part_duration = seed_duration as f32 / 8.0;

        let (current_seed_index, current_seed_instant) = match self.current_seed_info {
            None => {
                let current_seed_info = (0, Instant::now());
                self.current_seed_info = Some(current_seed_info);
                current_seed_info
            }
            Some(current_seed_info)
                if current_seed_info.1.elapsed().as_millis() > seed_duration =>
            {
                let current_seed_info = (current_seed_info.0 + 1, Instant::now());
                self.current_seed_info = Some(current_seed_info);
                current_seed_info
            }
            Some(current_seed_info) => current_seed_info,
        };

        if current_seed_index >= self.seeds.len() {
            let min_location = self
                .seeds
                .iter()
                .map(|seed| seed.last().unwrap())
                .min()
                .unwrap();
            let height = SCREEN_HEIGHT as f32 / (self.seeds.len() as f32 * 1.1 + 2.0);
            for (index, seed) in self.seeds.iter().enumerate() {
                let color = if seed.last().unwrap() == min_location {
                    TextType::Glow(GlowColor::Gold)
                } else {
                    TextType::Glow(GlowColor::White)
                };
                let line_y = SCREEN_HEIGHT as f32 / 2.0
                    + (index as f32 - (self.seeds.len() as f32) / 2.0) as f32 * height as f32;

                graphics.draw_rectangle_image(
                    array_to_rectangle(square_at_position(Vector2::new(50.0, line_y), 20.0)),
                    &self.images[0],
                );

                text_manager.draw_text_align(
                    graphics,
                    40,
                    color,
                    (75.0, line_y),
                    seed[0].to_string(),
                    Align::Left,
                );

                graphics.draw_rectangle_image(
                    array_to_rectangle(square_at_position(
                        Vector2::new(SCREEN_WIDTH as f32 - 50.0, line_y),
                        20.0,
                    )),
                    &self.images.last().unwrap(),
                );

                text_manager.draw_text_align(
                    graphics,
                    40,
                    color,
                    (SCREEN_WIDTH as f32 - 75.0, line_y),
                    seed.last().unwrap().to_string(),
                    Align::Right,
                );
            }

            sleep(Duration::from_millis(100));
            return;
        }

        let seed_elapsed = current_seed_instant.elapsed().as_millis();
        let part_index = (seed_elapsed as f32 / part_duration).floor() as usize + 1;

        for index in 0..part_index {
            if index >= 8 {
                break;
            }
            let value = self.seeds[current_seed_index][index];

            graphics.draw_rectangle_image(
                array_to_rectangle(square_at_position(
                    Vector2::new(50.0, 50.0 + index as f32 * 50.0),
                    20.0,
                )),
                &self.images[index],
            );

            text_manager.draw_text_align(
                graphics,
                40,
                TextType::Glow(GlowColor::White),
                (75.0, 50.0 + index as f32 * 50.0),
                value.to_string(),
                Align::Left,
            );
        }

        if part_index < 8 {
            let map = &self.maps.get(part_index_to_string(part_index - 1)).unwrap();
            let value = self.seeds[current_seed_index][part_index - 1];

            text_manager.draw_text(
                graphics,
                40,
                TextType::Gray,
                (SCREEN_WIDTH as f32 / 2.0, SCREEN_HEIGHT as f32 / 2.0 - 70.0),
                format!("from {} to {}", map.from, map.to),
            );
            let height = (SCREEN_HEIGHT as f32 / 2.0) / (map.ranges.len() as f32 * 1.1 + 2.0);

            for (i, range) in map.ranges.iter().enumerate() {
                let line_y = SCREEN_HEIGHT as f32 / 2.0 + i as f32 * height as f32;

                let color =
                    if value >= range.source_start && value < range.source_start + range.length {
                        TextType::Glow(GlowColor::Gold)
                    } else {
                        TextType::Glow(GlowColor::White)
                    };

                text_manager.draw_text_align(
                    graphics,
                    25,
                    color,
                    (80.0, line_y),
                    range.source_start.to_string(),
                    Align::Left,
                );

                if value >= range.source_start && value < range.source_start + range.length {
                    let diff = range.destination_start - range.source_start;
                    let diff_text = if diff > 0 {
                        format!("+{diff}")
                    } else {
                        diff.to_string()
                    };
                    text_manager.draw_text(
                        graphics,
                        25,
                        color,
                        (SCREEN_WIDTH as f32 / 2.0, line_y),
                        diff_text,
                    );
                }
                text_manager.draw_text_align(
                    graphics,
                    25,
                    color,
                    (SCREEN_WIDTH as f32 - 80.0, line_y),
                    (range.source_start + range.length).to_string(),
                    Align::Right,
                );
            }
        }
    }
}

type Seed = Vec<i64>;

struct Section {
    from: &'static str,
    to: &'static str,
    ranges: Vec<Range>,
}

#[derive(Debug, Copy, Clone)]
struct Range {
    destination_start: i64,
    source_start: i64,
    length: i64,
}

fn main() {
    let (seeds, maps) = input().trim().split_once("\n\n").unwrap();

    let mut seeds: Vec<Seed> = seeds
        .trim()
        .split_once(": ")
        .unwrap()
        .1
        .split_whitespace()
        .map(|seed| vec![seed.parse().unwrap()])
        .collect();

    let maps: HashMap<&str, Section> = maps
        .split("\n\n")
        .map(|section| {
            let (header, ranges) = section.split_once("\n").unwrap();
            let mut header = header.trim_end_matches(" map:").split('-');
            let from = header.next().unwrap();
            header.next().unwrap();
            let to = header.next().unwrap();

            let ranges = ranges
                .lines()
                .map(|line| {
                    let mut numbers = line.trim().split_whitespace();

                    Range {
                        destination_start: numbers.next().unwrap().parse().unwrap(),
                        source_start: numbers.next().unwrap().parse().unwrap(),
                        length: numbers.next().unwrap().parse().unwrap(),
                    }
                })
                .collect();

            (from, Section { from, to, ranges })
        })
        .collect();

    for seed in seeds.iter_mut() {
        'seed_loop: while seed.len() < 8 {
            let index = seed.len();
            let value = *seed.last().unwrap();
            for range in &maps.get(part_index_to_string(index - 1)).unwrap().ranges {
                if value >= range.source_start && value < range.source_start + range.length {
                    seed.push(value + (range.destination_start - range.source_start));
                    continue 'seed_loop;
                }
            }

            seed.push(value);
        }
    }

    run(MyState {
        seeds,
        maps,
        current_seed_info: None,
        images: vec![],
    });
}

fn part_index_to_string(index: usize) -> &'static str {
    match index {
        0 => "seed",
        1 => "soil",
        2 => "fertilizer",
        3 => "water",
        4 => "light",
        5 => "temperature",
        6 => "humidity",
        7 => "location",
        _ => panic!(),
    }
}

fn ex_input() -> &'static str {
    "
seeds: 79 14 55 13

seed-to-soil map:
50 98 2
52 50 48

soil-to-fertilizer map:
0 15 37
37 52 2
39 0 15

fertilizer-to-water map:
49 53 8
0 11 42
42 0 7
57 7 4

water-to-light map:
88 18 7
18 25 70

light-to-temperature map:
45 77 23
81 45 19
68 64 13

temperature-to-humidity map:
0 69 1
1 0 69

humidity-to-location map:
60 56 37
56 93 4
    "
}

fn input() -> &'static str {
    "
seeds: 1482445116 339187393 3210489476 511905836 42566461 51849137 256584102 379575844 3040181568 139966026 4018529087 116808249 2887351536 89515778 669731009 806888490 2369242654 489923931 2086168596 82891253

seed-to-soil map:
3356468240 2934525445 29117552
4275689831 4042213712 19277465
949730239 1589971986 381295142
2205130246 3387543719 106537240
2442849314 2188173171 261901063
2027919967 875104547 177210279
4258838211 4278115676 16851620
1969509044 3125327238 8268732
3602491835 652291761 28146990
3630638825 3122528592 2798646
1725486280 3012647256 109881336
3232765106 192460045 36910273
4042213712 4061491177 216624499
2311667486 3256361891 131181828
2849273982 3133595970 102505596
1365732141 2963642997 49004259
3093408594 3494080959 139356512
3385585792 1971267128 216906043
2954083526 56695294 82629774
1331025381 2483732118 34706760
3322810356 2450074234 33657884
3269675379 139325068 53134977
2704750377 680438751 144523605
1977777776 824962356 50142191
929469914 3236101566 20260325
0 1363064706 224603332
1835367616 2800384017 134141428
647524775 2518438878 281945139
2951779578 1587668038 2303948
1414736400 1052314826 310749880
224603332 229370318 422921443
3036713300 0 56695294

soil-to-fertilizer map:
1496362907 858318422 178940893
2630847538 53056663 113045954
2283611281 2088524569 46779241
3913589699 3841520880 84292875
2330390522 2567432522 113508448
2524251705 2460836689 106595833
3063003021 2963042999 10035541
3857182342 4222898477 56407357
37310480 2913919563 49123436
740077448 300637488 194492816
1474645818 1456131667 21717089
934570264 166102617 54182136
988752400 1796958285 94278756
1717132707 1928547521 159977048
521508712 2758310702 26883128
3092599643 495130304 257125345
1877109755 2785193830 128725733
3997882574 3925813755 297084722
548391840 2269151081 191685608
3841520880 4279305834 15661462
86433916 2187553774 81597307
1675303800 816489515 41828907
2443898970 220284753 80352735
2005835488 2973078540 277775793
0 1891237041 37310480
1299192340 1390736804 45833782
1345026122 2680940970 77369732
168031223 1037259315 353477489
1083031156 3250854333 138997462
3073038562 1436570586 19561081
1275085281 792382456 24107059
1222028618 0 53056663
3349724988 752255649 40126807
2743893492 1477848756 319109529
1422395854 2135303810 52249964

fertilizer-to-water map:
4238460975 3150676058 14156194
4014738493 2552067322 165315151
2782663538 3067003586 60442604
718350022 1496692875 242681298
0 662267357 48987302
73802866 465780476 196486881
270289747 736070223 448060275
2501821195 4263593575 31373721
961031320 0 47107691
2448671317 2498917444 53149878
2843106142 3164832252 729755546
4180053644 2717382473 58407331
2533194916 4223942180 39651395
48987302 711254659 24815564
2037107882 2799681618 267321968
2424779503 2775789804 23891814
1547175259 1304493961 192198914
4252617169 4084472268 19120259
1871165319 3918529705 61914957
3572861688 1871165319 441876805
2758721631 3894587798 23941907
2304429850 4103592527 120349653
1933080276 3980444662 104027606
1008139011 47107691 418672785
1426811796 1184130498 120363463
4271737428 3127446190 23229868
2572846311 2313042124 185875320

water-to-light map:
1985898327 3318267441 140753926
1347635148 2083526793 398378839
3514250773 2747115878 571151563
2126652253 1347635148 86837038
3336431644 3459021367 177819129
2927342235 1508339551 335222044
0 365247178 425818944
500722155 917309008 28280043
580341873 0 365247178
3262564279 1434472186 73867365
2662131989 2481905632 265210246
2213489291 3636840496 415606487
529002198 791066122 51339675
1746013987 1876597806 206928987
2629095778 1843561595 33036211
425818944 842405797 74903211
1952942974 4052446983 32955353

light-to-temperature map:
2521168614 3718558727 45222681
2372021437 4250929390 44037906
2416059343 3070381062 105109271
391082070 1490595758 135161830
2750033935 3567996322 26024928
2631208948 4085216210 118824987
1606793146 1161017018 154561777
0 27318229 64007187
2566391295 3763781408 64817653
2205452704 2073181756 31511904
2354729618 109736771 15352358
526243900 143079078 467881514
165490760 1625757588 156087087
321577847 610960592 69504223
1116662502 1843304861 180171121
2173847890 2322708438 31604814
64007187 1315578795 101483573
1315244978 1417062368 73533390
2989245773 2722605383 134588769
3800621948 4204041197 46888193
3287840442 2372021437 350583946
1067967658 12382058 14936171
3276570277 3971207241 11270165
3152032800 3594021250 124537477
1761354923 2023475982 49705774
2236964608 680464815 117765010
4152595905 3982477406 102738804
3847510141 3920328030 50879211
3638424388 3429696886 70468591
1098672553 125089129 17989949
1999021216 798229825 174826674
4255334709 3528363735 39632587
1296833623 91325416 18411355
3708892979 3828599061 91728969
1388778368 2104693660 218014778
1811060697 973056499 187960519
1082903829 2354313252 15768724
2776058863 2857194152 213186910
3898389352 3175490333 254206553
994125414 1781844675 61460186
3123834542 3500165477 28198258
1055585600 0 12382058

temperature-to-humidity map:
3270313314 2596058682 35302332
4226607799 1853648898 10548841
2997401183 2994049594 173176623
3460843740 3377888646 164140301
762069527 4252128205 42839091
3170577806 1535455627 99735508
356504730 1864197739 73760842
1176796835 3767577945 398679053
2184102927 1268190151 103654223
1839964412 1995769237 103314277
623505847 2855485914 138563680
551373625 970847216 72132222
2605053483 2099083514 149971726
804908618 510961861 330954039
2476122167 841915900 128931316
3809356495 2631856914 172503566
3981860061 3167226217 210662429
254072453 98437479 41846688
3305615646 1164087491 104102660
98437479 140284167 137804809
236242288 278088976 17830165
2939640029 1477694473 57761154
2390051063 2249055240 86071104
2755025209 3582963125 184614820
1135862657 3542028947 40934178
4237156640 1937958581 57810656
3409718306 2804360480 51125434
1943774589 4166256998 85871207
2029645796 356504730 154457131
3624984041 1827821728 25827170
430265572 1042979438 121108053
1575475888 1371844374 3556186
4192522490 1635191135 34085309
3650811211 1669276444 158545284
1943278689 2631361014 495900
1579032074 2335126344 260932338
2287757150 1375400560 102293913

humidity-to-location map:
4260564640 3164238850 33008819
2293789713 3286584985 52546193
2087002602 2864270962 68938922
1297747555 1309838844 89337809
3093628267 3842203176 155987450
2609276317 3498417185 343785991
658125616 1701481170 20754060
1593540119 1399176653 128695111
2283933279 2244808425 9856434
3849705959 3012295008 151943842
678879676 1170609407 139229437
1009204170 485451665 270016861
2155941524 4227835566 67131730
4032228982 4161145047 66690519
3532191685 2179585888 65222537
2953062308 2268317830 85579399
2252523457 3339131178 23090374
818109113 755468526 17485651
4098919501 2254664859 13652971
3379699400 2416907530 152492285
4293573459 2662947015 1393837
835594764 1527871764 173609406
2275613831 3422250936 8319448
4112572472 2933209884 24098564
1279221031 466925141 18526524
3038641707 2957308448 54986560
397655230 0 260470386
2223073254 3392800733 29450203
0 772954177 397655230
4136671036 3430570384 30346404
3597414222 3998190626 162954421
2546266016 2353897229 63010301
3249615717 3460916788 37500397
1387085364 260470386 206454755
3287116114 2087002602 92583286
3760368643 3197247669 89337316
2346335906 2664340852 199930110
4167017440 2569399815 93547200
4001649801 3362221552 30579181
    "
}
