#import "../lib.typ": *

#show heading.where(level: 2): set heading(outlined: false)

= Introduction

== Contexte et problématique
La conteneurisation s'est imposée comme le mode standard de déploiement des
applications dans les environnements modernes. Les outils qui gravitent autour
d'elle, tels que Docker Compose @bib-docker-compose ou Kubernetes
@bib-kubernetes, adoptent un mode d'opération déclaratif dans lequel
l'utilisateur décrit l'état désiré et le système se charge de le maintenir. Ce
paradigme a également gagné l'infrastructure avec des outils tels que Terraform
@bib-terraform, permettant de gérer l'infrastructure sous forme de code (IaC)
@bib-ibm-iac plutôt qu'au moyen de procédures manuelles. Le modèle GitOps
@bib-gitops pousse cette logique plus loin, en faisant d'un dépôt Git l'unique
source de vérité de la configuration ainsi déclarée, toute modification de
l'infrastructure ou des applications déployées transitant par un commit
versionné dans ce dépôt plutôt que par une action directe sur le système cible.
Il s'agit d'une partie importante de la mise en œuvre des pratiques DevOps et de
CI/CD. Dans ces approches, l'interaction directe avec le système d'exploitation
devient l'exception plutôt que la règle.

Toutefois, les systèmes d'exploitation sous-jacents reposent encore
majoritairement sur des distributions génériques, administrées avec des outils
de gestion de configuration dont le modèle d'exécution reste fondamentalement
impératif, ce qui complique l'automatisation et la reproductibilité. De plus,
ces distributions n'intègrent pas la conteneurisation comme un élément de
première classe du système et nécessitent de configurer et gérer les conteneurs
de manière distincte du système d'exploitation. De ce fait, la complexité de la
configuration et du maintien en état repose entièrement sur l'administrateur.

Certaines solutions plus spécialisées existent, alliant déclarativité et support
natif pour la conteneurisation, mais elles souffrent de lacunes diverses. NixOS
@bib-nix permet une gestion déclarative de l'ensemble du système, hôte et
conteneurs, mais cette déclarativité reste ponctuelle: l'état désiré est
appliqué une seule fois, à l'exécution d'une commande, sans qu'aucune boucle de
contrôle ne surveille le système ni n'en corrige les dérives susceptibles de
survenir par la suite. D'autres solutions, telles que Talos Linux @bib-talos,
permettent une administration entièrement déclarative et continue, mais
s'intègrent étroitement avec Kubernetes, au prix d'une complexité opérationnelle
et d'une empreinte mémoire disproportionnées pour le simple déploiement de
quelques conteneurs.

Il manque donc un système d'exploitation dans lequel il est possible de décrire
la configuration de l'hôte et des conteneurs dans un modèle déclaratif unique et
homogène, et capable de maintenir cet état de manière continue et autonome. Un
tel système serait particulièrement pertinent pour les déploiements modestes ou
embarqués, où la complexité d'un orchestrateur complet n'est pas justifiée, mais
où il est néanmoins souhaitable de disposer d'un système déclaratif.

== Objectifs
L'objectif central est de fournir un système d'exploitation entièrement
configurable selon un modèle déclaratif unique, dans lequel la configuration de
l'hôte et celle des conteneurs sont intégrées de manière homogène. La
conteneurisation est traitée comme un élément natif du système: les conteneurs
sont décrits et gérés au même titre que les autres ressources, sans couche
externe. En particulier, le système doit maintenir l'état désiré de manière
continue et autonome, en s'appuyant sur une boucle de contrôle qui surveille en
permanence l'état réel, détecte automatiquement les dérives et les corrige sans
intervention humaine.

Parce que le système est destiné à des déploiements exposés à Internet et
fonctionne sans surveillance constante, la sécurité est une exigence
primordiale. Elle se concrétise par une isolation forte des composants système
entre eux, une réduction maximale de la surface d'attaque et l'absence de tout
accès interactif direct (notamment pas d'accès SSH). Le mode d'interaction
entièrement déclaratif et automatisé permet de rendre cette absence d'accès
direct viable.

Pour garantir une empreinte mémoire minimale et une maîtrise complète sur le
fonctionnement du système, la solution est construite directement au-dessus du
noyau Linux @bib-linux-kernel, sans recourir à une distribution préexistante.
Elle comprend un processus d'initialisation (PID 1), un environnement
utilisateur (user-space) restreint (sans shell ou systemd) qui gère le système,
et intègre uniquement le runtime de conteneurs et un bootloader comme composants
externes #footnote[
    Par "composant externe", il est entendu ici un logiciel tiers fonctionnant
    comme un service ou un processus indépendant, par opposition aux
    bibliothèques logicielles (souvent appelés packages ou librairies).
]. Ce choix de conception permet de réduire la complexité superflue et de
limiter encore la surface d'attaque.

== Cadre du travail et méthodologie
Ce travail s'inscrit dans l'obtention du titre de Bachelor of Science en
Informatique et systèmes de communication, orientation Informatique logicielle,
à la Haute école du paysage, d'ingénierie et d'architecture de Genève (HEPIA).
La problématique abordée est issue d'une motivation personnelle, née de
l'expérience dans l'administration de petites infrastructures, où le maintien
manuel de systèmes hétérogènes et l'absence de correction automatique des
dérives de configuration ont motivé la recherche d'une solution plus robuste.
Réalisé à plein temps du 11 mai au 19 août 2026, il reflète l'état des
connaissances et des technologies à cette période. Il a été précédé du projet de
semestre, disponible sur
https://gitedu.hesge.ch/flg_bachelors/ps/2025/container_os, qui a permis une
analyse détaillée de la problématique, l'identification des briques techniques
nécessaire et l'établissement des premiers éléments conceptuels et techniques.
L'ensemble du code source produit est disponible sur le dépôt Git institutionnel
à l'adresse suivante:
https://gitedu.hesge.ch/flg_bachelors/tb/2026/container-infrastructure-deployment-os.
L'état du dépôt au moment de la publication du présent document est disponible
sur le tag #repo("", [`v0.0.0-dev.3.thesis`]) tandis que le dernier code testé
est disponible sur la branche `main`.

Des intelligences artificielles (IA) génératives ont été utilisées de manière
ponctuelle pour améliorer la qualité rédactionnelle; un texte de base contenant
l'intégralité du fond et de l'organisation a toujours été fourni en amont. Pour
le code, elles ont principalement servi d'outil d'analyse, en complément des
outils d'analyse de code traditionnels, afin de détecter d'éventuelles failles,
bogues ou usages contraires aux bonnes pratiques. L'IA a également été employée
pour conforter l'exhaustivité des recherches documentaires et des efforts de
débogage: une fois la collecte humaine jugée suffisante ou qu'elle se heurtait à
une impasse, l'IA a été interrogée pour signaler d'éventuels angles morts, en
suggérant des mots-clés ou des références complémentaires. Ces suggestions ont
toujours été vérifiées avant d'être utilisées. L'IA n'a explicitement pas été
utilisée dans le cadre de la production de code, pour prendre des décisions
architecturales ou organisationnelles relatives au code, ainsi que pour
effectuer recherches et débogages initiaux. Un compte rendu détaillé et
exhaustif de ces usages figure au sein de l'#appendix-full-ref(
    <appendix-ai>,
).

Le travail a été structuré en trois jalons principaux. Une première phase
conceptuelle, jusqu'à la fin mai, a permis de poser les bases technologiques.
Une phase de développement a suivi jusqu'à fin juin, intégrant l'ensemble des
fonctionnalités strictement nécessaires à l'obtention d'une solution testable et
comparable répondant aux objectifs techniques de l'énoncé. Enfin,
l'implémentation de fonctionnalités supplémentaires et l'amélioration continue
de la solution ont occupé la période allant jusqu'à fin juillet. Le déroulé
exact des différentes étapes est disponible dans le #chapter-full-ref(
    <results-academic>,
).

== Structure du document
Le chapitre #chapter-num-ref(<ch:functional-overview>) présente le système du
point de vue de l'utilisateur, sans détail d'implémentation. Le
#chapter-num-ref(<ch:system-design>), consacré à la conception du système,
expose les décisions architecturales conceptuelles majeures. Le chapitre
#chapter-num-ref(<ch:implementation>) détaille ensuite les choix techniques
retenus pour la réalisation de cette architecture, ainsi que les détails
d'implémentation importants. Le chapitre #chapter-num-ref(<ch:validation>)
présente la démarche de validation adoptée, ainsi que l'analyse de performance,
dont les résultats sont ensuite comparés au #chapter-num-ref(<ch:comparison>)
avec NixOS et Talos Linux. Le #chapter-num-ref(<ch:results>) dresse le bilan du
travail réalisé et en discute les limites et perspectives.
