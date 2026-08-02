#import "../lib.typ": *
#import "../../packages.typ": *
#import packages.lilaq as lq
#import "../data/time-noinstall.typ": delta
#import "../data/time-noinstall.typ" as iso
#import "../data/time-install.typ" as disk

#let lim = calc.max(
    ..delta(iso.time_to_run_container, iso.time_to_kernel),
    ..delta(
        disk.time_to_run_container,
        disk.time_to_kernel,
    ),
)
#let lim = calc.ceil(lim) * 1.1

#figure(
    label: <val-boot-time>,
    caption: [
        Chronologie des étapes de démarrage jusqu'à l'exécution d'un conteneur
    ],
    note: [
        Temps absolu en secondes mesuré à partir du démarrage du noyau,
        après le bootloader, des différentes étapes du cycle de vie sur un
        échantillon de 100.

        La mesure depuis une installation sur disque sont représentée sur le
        plan supérieur, tandis que les mesures depuis l'image ISO (mode
        éphémère) sont représentée sur le plan inférieur.
    ],
    source: made-by-self,
    lq.diagram(
        width: 100%,
        xlabel: [Temps \[s\]],
        height: 6cm,
        xaxis: (
            exponent: 0,
            lim: (0, lim),
        ),
        yaxis: (
            ticks: none,
        ),

        /// DISK
        lq.hviolin(
            delta(
                disk.time_to_init + disk.time_to_init_post,
                disk.time_to_kernel + disk.time_to_kernel_post,
            ),
            trim: false,
            label: [Time until /init],
            side: "high",
            color: green,
        ),
        lq.hviolin(
            delta(
                disk.time_to_dhcp + disk.time_to_dhcp_post,
                disk.time_to_kernel + disk.time_to_kernel_post,
            ),
            trim: false,
            label: [Time until DHCP route received],
            side: "high",
            color: blue,
        ),
        lq.hviolin(
            delta(disk.time_to_downloading_image, disk.time_to_kernel),
            trim: false,
            label: [Time until image downloading],
            side: "high",
            color: teal,
        ),
        lq.hviolin(
            delta(disk.time_to_run_container_post, disk.time_to_kernel_post),
            trim: false,
            side: "high",
            color: purple,
            label: [Time until container started (no pull)],
        ),
        lq.hviolin(
            delta(disk.time_to_run_container, disk.time_to_kernel),
            label: [Time until container started (pull)],
            side: "high",
            color: red,
        ),

        /// ISO
        lq.hviolin(
            delta(iso.time_to_init, iso.time_to_kernel),
            trim: false,
            side: "low",
            color: green,
        ),
        lq.hviolin(
            delta(iso.time_to_dhcp, iso.time_to_kernel),
            trim: false,
            side: "low",
            color: blue,
        ),
        lq.hviolin(
            delta(iso.time_to_downloading_image, iso.time_to_kernel),
            trim: false,
            side: "low",
            color: teal,
        ),
        lq.hviolin(
            delta(iso.time_to_run_container, iso.time_to_kernel),
            side: "low",
            color: red,
        ),
    ),
)
