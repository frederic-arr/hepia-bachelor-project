#import "../lib.typ": *

= Comparaison avec les solutions existantes

== Critères

=== Automatisation

Le cycle de vie de la solution doit être entièrement automatisable, dès la phase
d'installation. Plus spécifiquement, l'installation, la configuration initiale
et le déploiement de l'hôte doivent pouvoir être réalisés à distance, de manière
programmatique et sans intervention humaine. Ce critère implique que tous les
artefacts nécessaires au déploiement de la solution, à l'exception des fichiers
de configuration, soient déjà disponibles sous leur forme finale. Dans le cadre
de cette évaluation, les approches reposant uniquement sur des mécanismes
d'initialisation dépendants de l'environnement sont écartées, car elles
supposent que la configuration soit injectée dès l'installation initiale, mise à
disposition par un service externe ou intégré à l'artefact déployé.

=== Légèreté

#todo[Légèreté][
    - Parler de l'empreinte mémoire requise + disponible
    - Cible: pouvoir démarrer sur 512 MiB de RAM
]

=== Rapidité

#todo[Rapidité][
    - Parler de l'empreinte mémoire requise + disponible
    - Cible: pouvoir démarrer sur 512 MiB de RAM
]

=== Simplicité

La solution doit être aussi simple que possible à l'utilisation. La
configuration doit être centralisée, clairement structurée et facilement
modifiable par une personne ayant des connaissances dans le domaine concerné. La
mise à jour ne doit pas exiger de procédures fastidieuses, risquées ou
nécessitant des compétences spécialisées autres que celles liées au domaine
configuré.

== NixOS

NixOS @bib-nix est une distribution basée sur le gestionnaire Nix, qui permet la
création de systèmes reproductibles selon une approche déclarative. Nix cherche
à simplifier le partage et la reproduction des configurations et reste avant
tout générique. Toutefois, la gestion des conteneurs est intégrée à NixOS et
cette solution tend à être l'une des plus recommandées pour créer des parcs de
machines déclaratives.

L'évaluation de la solution a été réalisée sur la version 25.11, qui utilise la
version 6.12.62 du noyau Linux, publiée le 30 novembre 2025. Cette version est
disponible sous la licence MIT.

#todo[NixOS][
    - Parler de l'empreinte mémoire requise + disponible
    - Parler du temps d'installation et de démarrage
    - Parler du workflow et du fichier de configuration
]

// Nixos 200MiB de RAM boot en 13s MAIS c'est des conditions très optimales etc pour le temps de boot
// Aussi dedans le temps de build n'est pas compris, et il est considérable pour télécharger tous les packets et build l'image NixOS (puisque c'est nous qui devons le faire)

== Talos Linux

Talos Linux @bib-talos a été conçu dès le départ comme un système d'exploitation
sécurisé, immuable et minimaliste afin de faciliter la gestion de _clusters_
Kubernetes. Son objectif est d'éliminer les dérives de configuration,
c'est-à-dire les écarts progressifs entre l'état réel du système et sa
configuration attendue, en traitant la configuration du système comme du code
déclaratif. Toute la gestion du système s'effectue au travers d'une API gRPC et
d'un client en ligne de commande. De plus, Talos Linux cherche à réduire le
risque de dépendance à un fournisseur _cloud_ particulier en proposant une
distribution pouvant s'exécuter dans différents environnements de manière
homogène.

L'évaluation a porté sur la version 1.11.5, qui utilise la version 6.12.52 du
noyau Linux, et a été publiée le 6 novembre 2025. Ce logiciel est sous la
licence MPL-2.0.

#todo[Talos Linux][
    - Parler de l'empreinte mémoire requise + disponible
    - Parler du temps d'installation et de démarrage
    - Parler du workflow et du fichier de configuration
]

== Synthèse

// TODO

// @typstyle off
#{
  set text(size: 10pt)
  show table.cell.where(y: 0): set text(weight: "bold")
  show table.cell.where(x: 0): set text(weight: "bold")

  let mkcell(fill: none, default: none) = {
    return (..args) => {
      if args.pos().len() == 0 {
        table.cell(fill: fill, default)
      } else {
        table.cell(fill: fill, args.pos().at(0))
      }
    }
  }

  let y = mkcell(fill: green.transparentize(70%), default: sym.checkmark)
  let n = mkcell(fill: red.transparentize(70%), default: sym.crossmark)
  let o = mkcell(fill: gray.transparentize(70%), default: sym.nothing)
  let w = mkcell(fill: orange.transparentize(70%), default: sym.star)


  figure(
    label: <fig-sota-comp>,
    caption: [Comparaison des solutions existantes],
    note: [
      \ #sym.checkmark signifie que le critère est atteint
      \ #sym.crossmark signifie que le critère n'est pas atteint
    ],
    table(
      columns: (auto, 1fr, 1fr, 1fr),
      table.header([Critères],           [ContainerOS],  [NixOS],       [Talos]),
      [Dernière date],                   o[N/A],         o[2025-11-30], o[2025-11-06],
      [Version évaluée],                 o[N/A],         o[25.11],      o[1.11.5],
      [Version du kernel],               o[6.18],        o[6.12],       o[6.12],
      [Licence],                         o[N/A],         o[MIT],        o[MPL-2.0],
      table.hline(stroke: 2pt + black),
      [Automatisation],                  y(),            n(),           y(),
      [Mémoire requise],                 y[160 MiB],     w[200MiB],     n[1.3 GiB],
      [Rapidité d'installation],         y[36s],         [4mn],         [98s],
      [Rapidité de démarrage],           y[5.6s],        [13s],         [98s],
      [Simplicité],                      y(),            n(),           n(),
    )
  )
}

// TODO
