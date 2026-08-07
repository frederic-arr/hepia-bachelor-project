#import "../lib.typ": *

= Le langage Nix <appendix:nix-primer>
Cette annexe présente les concepts fondamentaux de Nix utilisés dans le système
de build présenté au #chapter-full-ref(<ch:implementation:build>). Le contenu
est largement inspiré de
https://linuxvox.com/blog/an-overview-of-nix-os-architecture/

== Dérivations
Dans Nix, une dérivation représente une fonction pure (sans effet de bord et ne
se basant que sur ses entrées), accompagnée des instructions nécessaires pour
build un logiciel depuis ses sources afin de créer un artefact entièrement
autonome, ou "self-contained" (l'ensemble des dépendances nécessaires à
l'utilisation de l'artefact est inclus avec celui-ci, à la manière d'un
conteneur). Les dérivations sont "évaluées", c'est-à-dire exécutées, dans un
environnement complètement isolé. En outre, les dérivations étant des fonctions
pures, produisant, pour des entrées identiques, des sorties identiques au bit
près, leur résultat est mis en cache, de sorte que deux évaluations d'une même
dérivation ne soient en réalité exécutées qu'une seule fois.

== nixpkgs
Nixpkgs est le dépôt officiel de Nix, dans lequel les paquets sont mis à
disposition sous forme de dérivations déjà construites et mises en cache; il
s'agit de l'équivalent d'un dépôt APT, par exemple.

== Le Nix Store
Le Nix Store est le répertoire unique dans lequel l'ensemble des résultats des
dérivations Nix sont placés: `/nix/store`. Chaque dérivation y occupe un
sous-répertoire dont le nom intègre un hash cryptographique dérivé des entrées
de la dérivation correspondante, ce qui garantit qu'aucune dérivation ne partage
de chemin avec une autre. Cette unicité des chemins permet à plusieurs versions
d'un même logiciel de coexister sans jamais s'écraser mutuellement, et les
dérivations y sont ajoutées ou retirées en tant qu'unités complètes, ce qui
exclut toute installation partielle.

== Nix comme système de build
En tant que système de build, Nix ne se limite pas au build d'artefacts isolés:
il permet également de définir des environnements de développement
reproductibles, activables au moyen de la commande `nix develop`, ainsi que
d'exécuter une commande arbitraire au sein d'un tel environnement sans l'ouvrir
de manière interactive, au moyen de `nix develop -c <commande>`. Cette dernière
capacité permet à un pipeline CI/CD de bénéficier exactement du même
environnement que celui utilisé en développement local.
