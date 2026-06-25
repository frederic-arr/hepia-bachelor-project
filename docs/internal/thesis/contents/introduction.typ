#import "../lib.typ": *

#show heading.where(level: 2): set heading(outlined: false)
#show link: set text(blue)
#show link: underline

= Introduction

== Contexte et problématique

#todo-inline[Ajouter les références aux divers outils mentionnés.]

La conteneurisation s'est imposée comme le mode standard de déploiement des
applications dans les environnements modernes. Les outils qui gravitent autour
d'elle, tels que Docker Compose ou Kubernetes, adoptent un mode d'opération
déclaratif dans lequel l'utilisateur décrit l'état désiré et le système se
charge de le maintenir. Ce paradigme a également gagné l'infrastructure cloud
avec des outils tels que Terraform. Il a donné naissance au modèle GitOps, qui
consiste à stocker l'ensemble de la configuration dans Git. Ainsi, chaque fois
qu'une mise à jour est effectuée, elle est appliquée automatiquement, ce qui
rend le déploiement et l'infrastructure entièrement automatisés. Dans ces
approches, l'interaction directe avec le système d'exploitation devient
l'exception plutôt que la règle.

Toutefois, les systèmes d'exploitation sous-jacents reposent encore
majoritairement sur des distributions génériques, administrées avec des outils
de gestion de configuration dont le modèle d'exécution reste fondamentalement
impératif, ce qui complique l'automatisation et la reproductibilité. De plus,
ces distributions ne considèrent pas la conteneurisation comme un élément
intégral au système et nécessitent de configurer et gérer les conteneurs de
manière distincte du système d'exploitation. De ce fait, la complexité de la
configuration et du maintien en état repose entièrement sur l'administrateur.

Certaines solutions plus spécialisées existent, alliant déclarativité et support
natif pour la conteneurisation, mais elles souffrent de lacunes diverses. NixOS
permet une gestion déclarative de l'ensemble du système, hôte et conteneurs,
mais cette déclarativité est ponctuelle: l'état désiré n'est appliqué qu'à un
moment donné, et aucune boucle de contrôle ne corrige les dérives ultérieures.
Flatcar Container Linux offre une base minimale et orientée conteneurs, mais ne
rend déclaratif que la phase d'installation initiale de l'hôte. D'autres
solutions, telles que Talos Linux, permettent une administration entièrement
déclarative et continue, mais s'intègrent étroitement avec Kubernetes, au prix
d'une complexité opérationnelle et d'une empreinte mémoire disproportionnées
pour le simple déploiement de quelques conteneurs.

Il manque donc un système d'exploitation capable de décrire la configuration de
l'hôte et des conteneurs dans un modèle déclaratif unique et homogène, et de
maintenir cet état de manière continue et autonome. Un tel système serait
particulièrement pertinent pour les déploiements modestes, où la complexité d'un
orchestrateur complet n'est pas justifiée, mais où l'on souhaite néanmoins
disposer d'un système déclaratif capable de se maintenir en état de manière
autonome.

== Objectifs

Le travail de semestre effectué préalablement a permis d'analyser en profondeur
les forces et les faiblesses des différentes solutions disponibles. Il a
débouché sur la présentation d'une architecture de très haut niveau, explorant
les concepts fondamentaux et les briques techniques nécessaires à
l'implémentation d'une solution adaptée. Le présent travail reprend ces
conclusions afin de concevoir une architecture détaillée, puis de l'implanter,
de la valider et de la comparer à Talos Linux et NixOS.

L'objectif central est de fournir un système d'exploitation entièrement
configurable selon un modèle déclaratif unique, dans lequel la configuration de
l'hôte et celle des conteneurs sont intégrées de manière homogène. La
conteneurisation est traitée comme un élément natif du système: les conteneurs
sont décrits et gérés au même titre que les autres ressources, sans couche
externe. En particulier, le système doit maintenir l'état déclaré de manière
continue et autonome, en s'appuyant sur une boucle de contrôle qui surveille en
permanence l'état réel, détecte automatiquement les dérives et les corrige sans
intervention humaine.

Parce que le système est destiné à des déploiements exposés à Internet et
fonctionne sans surveillance constante, la sécurité est une exigence
primordiale. Elle se concrétise par une isolation forte des composants système
entre eux, une réduction maximale de la surface d'attaque et l'absence de tout
accès interactif direct (notamment pas d'accès SSH). Le mode d'interaction
entièrement déclaratif et automatisé permet de rendre cette posture viable.

Pour garantir une empreinte mémoire minimale et une maîtrise complète sur le
fonctionnement du système, la solution est construite directement au-dessus du
noyau Linux, sans recourir à une distribution préexistante. Elle comprend un
processus d'initialisation (PID 1), un environnement utilisateur (user-space)
restreint (sans shell ou systemd) qui gère le système, et intègre uniquement le
runtime de conteneurs et un bootloader comme composants externes #footnote[
    Par "composant externe", il est entendu ici un logiciel tiers fonctionnant
    comme un service ou un processus indépendant, par opposition aux
    bibliothèques logicielles (souvent appelés packages ou librairies).
]. Ce choix de conception permet de réduire la complexité superflue et de
limiter encore la surface d'attaque.

#todo-inline[
    compléter la liste des "composants externes" si on en inclut d'autres
]

== Cadre du travail et méthodologie

Ce travail s'inscrit dans l'obtention du titre de Bachelor of Science en
Informatique et systèmes de communication, orientation Informatique logicielle,
à la Haute école du paysage, d'ingénierie et d'architecture de Genève (HEPIA).
Réalisé à plein temps du 11 mai au 19 août 2026, il reflète l'état des
connaissances et des technologies à cette période. Il a été précédé d'un travail
de semestre qui a permis une analyse détaillée de la problématique et
l'établissement des premiers éléments conceptuels et technique. L'ensemble du
code source produit est disponible sur le dépôt Git institutionnel à l'adresse
suivante:
https://gitedu.hesge.ch/flg_bachelors/tb/2026/container-infrastructure-deployment-os.

Des intelligences artificielles (IA) génératives ont été utilisées de manière
ponctuelle pour améliorer la qualité rédactionnelle; un texte de base contenant
l'intégralité du fond et de l'organisation a toujours été fourni en amont. Pour
le code, elles ont principalement servi d'outil d'analyse qualitative, en
complément des outils d'analyse de code traditionnels, afin de détecter
d'éventuelles failles, bogues ou usages contraires aux bonnes pratiques. L'IA a
également été employée pour conforter l'exhaustivité des recherches
documentaires et des efforts de débugage: une fois la collecte humaine jugée
suffisante ou qu'elle se heurtait à une impasse, elle a été interrogée pour
signaler d'éventuels angles morts, en suggérant des mots-clés ou des références
complémentaires. Ces suggestions ont toujours été vérifiées avant d'être
utilisées. Toute contribution de l'IA à la production de code, aux décisions
architecturales ou organisationnelles, ainsi qu'aux recherches fondamentales,
est explicitement exclue. Un compte rendu détaillé et exhaustif de ces usages
figure en annexe #todo-inline[Référencer l'annexe.].

Le travail a été structuré en trois jalons principaux. Une première phase
conceptuelle, jusqu'à la fin mai, a permis de poser les bases technologiques.
Une phase de développement a suivi jusqu'à fin juin, intégrant l'ensemble des
fonctionnalités strictement nécessaires à l'obtention d'une solution testable et
comparable répondant aux objectifs techniques de l'énoncé. Enfin,
l'implémentation de fonctionnalités additionnelles et l'amélioration continue de
la solution ont occupé la période allant jusqu'à fin juillet.

== Structure du document

Le reste du document est organisé en six chapitres. Le premier chapitre présente
le système du point de vue de l'utilisateur: il décrit le mode d'interaction
déclaratif, le cycle de vie des conteneurs, le fonctionnement attendu de la
boucle de contrôle et les principes de configuration. Les notions nécessaires à
la compréhension de la solution y sont introduites, sans entrer dans les détails
internes. Le deuxième chapitre expose la conception détaillée. Il définit
l'architecture générale, l'organisation des composants, leurs responsabilités et
leurs interfaces. Les structures de données propres au modèle déclaratif et au
mécanisme de réconciliation y sont spécifiées. Le troisième chapitre traite de
l'implémentation. Il détaille les choix techniques (langage, bibliothèques,
structures de code), les algorithmes principaux et la manière dont la boucle de
contrôle a été réalisée. L'environnement de développement et les outils de test
sont également présentés dans ce chapitre. Le quatrième chapitre est consacré à
la validation. Il expose la méthodologie employée pour tester le système (tests
unitaires, tests d'intégration, scénarios de dérive) et synthétise les résultats
obtenus, notamment en matière de réactivité et de fiabilité du maintien d'état.
Le cinquième chapitre compare la solution à Talos Linux et NixOS. La
méthodologie de comparaison y est précisée, le protocole de mesure et analyse
des résultats. Enfin, le sixième chapitre dresse un bilan global, discute les
forces et les limites de l'approche, et esquisse des perspectives d'amélioration
et de travaux futurs.
