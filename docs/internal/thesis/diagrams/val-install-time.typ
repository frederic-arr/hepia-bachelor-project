#import "../lib.typ": *
#import "../../packages.typ": *
#import packages.lilaq as lq
#import "../data/time-install.typ": *

#let lim = calc.max(
    ..delta(
        time_to_run_container,
        time_to_config,
    ),
)
#let lim = calc.ceil(lim)

#figure(
    label: <val-install-time>,
    caption: [Durée d'installation et de démarrage du conteneur après
        installation],
    note: [
        Temps absolu en secondes mesuré à partir de la réception de la
        configuration jusqu'à l'installation du système, et temps entre le
        démarrage du noyau et l'exécution d'un conteneur, installation incluse,
        sur un échantillon de 100.
    ],
    source: made-by-self,
    lq.diagram(
        width: 100%,
        xlabel: [Temps \[s\]],
        legend: (
            position: top,
            dx: 10%,
        ),
        xaxis: (
            exponent: 0,
            lim: (0, lim),
        ),
        yaxis: (
            ticks: none,
        ),

        lq.hviolin(
            delta(time_to_install, time_to_config),
            label: [Time to install],
        ),
        lq.hviolin(
            delta(time_to_run_container, time_to_config),
            label: [Time until container started],
        ),
    ),
)
