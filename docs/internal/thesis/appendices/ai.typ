#import "../lib.typ": *

= Utilisation de l'intelligence artificielle <appendix-ai>
Le modèle utilisé pour l'ensemble des interactions documentées ci-dessous est
Claude Sonnet 4.6 Thinking, via Perplexity.

== Recherche d'options de configuration manquantes du noyau Linux
Le problème rencontré est que Podman ne parvient pas à démarrer correctement des
conteneurs, la `defconfig` du noyau ne contenant pas l'ensemble des
fonctionnalités requises. La majorité des options manquantes ont été identifiées
manuellement au préalable, mais les messages d'erreur obtenus lors de la
configuration de netavark ne se sont pas révélés suffisamment explicites pour
identifier les options restantes. Le fragment de configuration du noyau (limité
aux modifications apportées à la `defconfig`),le message d'erreur obtenu et du
contexte (`defconfig`, système embarqué, etc.), ont été transmis avec une
demande visant soit à identifier l'option manquante, soit à fournir des sources
pertinentes. En pratique, l'IA ne parvient pas à identifier l'option manquante,
mais oriente effectivement la recherche vers le wiki de Gentoo, à partir duquel
une autre page permet d'identifier une partie des options manquantes; seule
l'option `CONFIG_NFT_FIB_IPV6` demeure introuvable par ce biais, l'IA ne l'ayant
ni identifiée ni sourcée. Une recherche complémentaire, menée après cet échec,
aboutit finalement à un commentaire publié sur le dépôt GitHub de
podman-compose, fournissant l'option manquante. L'IA n'apporte, dans ce cas,
qu'un résultat mitigé.

== Amélioration du pipeline de build Nix pour Rust
Le problème rencontré est que, lors de la construction du workspace Rust au
moyen de Nix et de Crane, toute modification apportée à l'une des crates
entraîne la reconstruction de l'ensemble des crates, plutôt que des seules
crates qui en dépendent effectivement. La configuration du workspace Cargo ainsi
que les fichiers Nix ont été fourni, avec la description du problème. L'IA ne
parvient ni à identifier la cause, ni à proposer une solution fonctionnelle. La
cause réelle du problème tient au fait que l'ensemble des dépendances, y compris
locales, est déclaré dans le `Cargo.toml` du workspace, ce qui invalide
systématiquement la préparation des artefacts effectuée par Crane; le retrait de
ces dépendances du workspace, au profit d'une déclaration du chemin directement
dans chaque crate dépendante, résout le problème, moyennant l'ajout d'une option
`deps` à la fonction de build afin d'inclure ces crates dans les sources. L'IA
n'apporte, dans ce cas, aucune aide et constitue une perte de temps estimée à 30
minutes.

== Mise en place de tests d'intégration Rust isolés par namespace
Le problème rencontré tient à la nature des paquets concernés, qui interagissent
directement avec le noyau Linux, ce qui rend l'écriture de tests unitaires ou
d'intégration délicate: ces tests modifient le noyau "hôte" et risquent de
provoquer des dysfonctionnements, par exemple par l'ajout ou la suppression
d'interfaces réseau, ou la modification de `/etc/resolv.conf`. Aucun travail
préalable n'a été mené sur ce point. Les besoins ont été décrits et il a été
demandé d'identifier les crates pertinentes pour ces usages. L'IA identifie
effectivement une crate pertinente (`netsim` @bib-netsim), qui, bien que ne
correspondant pas exactement au besoin, permet de s'inspirer de son code source
pour produire une implémentation propre au projet. L'IA apporte, dans ce cas,
une aide effective, avec un gain de temps estimé à 15 minutes.
