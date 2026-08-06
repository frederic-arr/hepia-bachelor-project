#import "../lib.typ": *

= Implémentation <ch:implementation>
Ce chapitre présente les choix techniques retenus pour la réalisation du
système, ainsi que les éléments d'implémentation significatifs. Les composants
logiciels et les choix technologiques globaux au projet sont d'abord présentés,
avant que l'implémentation de l'orchestrateur et des différents contrôleurs ne
soit détaillée. L'API et ses clients d'administration sont ensuite décrits,
suivis du système de fichiers racine, du processus de démarrage, ainsi que de
l'installation et du chiffrement du système. Le chapitre se conclut par la
description du système de build, de la configuration du noyau, de la génération
de l'image finale du système, ainsi que de l'environnement de test et de la
pipeline d'intégration continue mise en place.

== Composants
=== Choix des logiciels tiers
Limine @bib-limine est retenu comme bootloader et Podman @bib-podman-repo comme
runtime de conteneurs, ces deux choix ayant été établis lors du travail de
semestre. Podman est retenu en raison de son bon fonctionnement en mode rootless
ainsi que de sa légèreté, supérieure à celle des solutions alternatives
considérées. Limine est de même retenu pour sa légèreté, tout en offrant un
ensemble de fonctionnalités suffisamment complet pour les besoins du système.

=== Choix du langage
Rust est choisi comme langage principal pour l'implémentation du projet. Ce
choix repose sur la nécessité de disposer d'un langage permettant d'effectuer
aisément des appels système, contrainte partagée par le C, le C++ et le Go,
langages également considérés. Le choix final se porte sur Rust en raison de la
maîtrise préalable de ce langage, acquise avant le début du projet. Le langage
Go est par ailleurs utilisé pour la partie du projet reposant sur Terraform, cet
usage étant imposé par une contrainte propre à cet outil plutôt que par un choix
indépendant. Un rappel sur les concepts essentiels de Rust utilisés dans le
reste de ce chapitre est présenté dans l'#appendix-num-ref(
    <appendix:rust-primer>,
).

=== Choix des bibliothèques
Le nombre de bibliothèques externes est volontairement restreint, dans le but de
réduire la surface d'attaque du système et de conserver une maîtrise complète
sur son comportement. Le projet, incluant tous les contrôleurs, ne compte ainsi
que 30 dépendances directes, hors dépendances propres aux tests, ce qui
représente un total de 174 dépendances transitives#footnote[
    Mesuré au moyen de la commande `cargo tree`.
]. La majorité de ces dépendances sont par ailleurs des dépendances standard
dans un environnement Rust telles que le système de journalisation, la gestion
d'erreur, l'asynchronicité, ou les interfaces pour les appels système.

La communication entre composants, détaillée dans le #chapter-full-ref(
    <ch:implementation:grpc>,
), repose sur gRPC @bib-grpc. Tonic @bib-tonic constitue, à ce jour, la seule
implémentation gRPC pour Rust suffisamment mature et maintenue pour cet usage;
son adoption impose en retour le recours à Tokio @bib-tokio comme runtime
asynchrone, ce dernier étant par ailleurs la bibliothèque d'exécution asynchrone
la plus largement adoptée au sein de l'écosystème Rust, ce qui limite le risque
lié à cette dépendance imposée. La sérialisation des messages échangés repose
sur Serde @bib-serde, bibliothèque de facto standard au sein de l'écosystème
Rust pour cet usage. Les bibliothèques restantes sont, pour leur part, propres à
un contrôleur particulier et sont présentées avec celui-ci.

=== Communication entre les composants <ch:implementation:grpc>
Le protocole gRPC est utilisé uniquement comme mécanisme de transport et de
définition des procédures exposées par chaque composant; la structure des
messages échangés n'est pas définie par ce protocole. Depuis sa ProtoBuf 3,
celui-ci ne permet plus de marquer un champ comme obligatoire, dans le but de
garantir la rétrocompatibilité entre un client et un serveur dont les versions
diffèrent~@bib-grpc-proto3-optional. Cette contrainte n'est toutefois pas
pertinente dans le cadre du présent système, l'ensemble des composants étant
déployé simultanément et implémenté en Rust: le recours systématique à des
champs optionnels y complexifierait inutilement le code. À la place, chaque
message ne contient ainsi qu'un unique champ nommé `raw`, correspondant à la
structure Rust concernée, sérialisée en JSON, plutôt que de reposer sur la
structure de message générée par gRPC.

// TODO?: Dire que ça serait mieux de juste pas utiliser gRPC, mais ça permet
// d'avoir quelques features utile comme les retry ou les connexion lazy?

Deux services sont définis. Le service `StateService` permet à un contrôleur de
communiquer vers l'orchestrateur, notamment afin de signaler la nécessité d'une
réconciliation en dehors du cycle normal:

#figure(
    caption: [Service gRPC pour l'orchestrateur],
    source: link("proto/containeros/state/v1/service.proto"),
    ```protobuf
    service StateService {
      rpc ReconcileNow(ReconcileNowRequest) returns (ReconcileNowResponse) {}
    }
    ```,
)

Ce service n'expose qu'une unique procédure, `ReconcileNow`, dont l'appel
notifie l'orchestrateur qu'une ressource donnée doit être réconciliée
immédiatement, sans attendre la prochaine échéance de la ressource. Cette
procédure est notamment invoquée par un contrôleur lorsqu'un changement d'état
externe à la spécification, tel qu'un événement survenant sur la ressource qu'il
gère, rend le dernier état observé de cette ressource obsolète.

Le service `ReconcilerService`, implémenté cette fois par chaque contrôleur et
appelé par l'orchestrateur, permet à ce dernier de déclencher la validation ou
la réconciliation d'une ressource:

#figure(
    caption: [Service gRPC pour les contrôleurs],
    source: link("proto/containeros/reconciler/v1/service.proto"),
    ```protobuf
    service ReconcilerService {
      rpc Validate(ValidateRequest) returns (ValidateResponse) {}
      rpc Reconcile(ReconcileRequest) returns (ReconcileResponse) {}
    }
    ```,
)

Ce service expose deux procédures distinctes, correspondant aux deux opérations
décrites dans les #chapters-full-ref(
    <ch:implementation:reconciliation>,
    <ch:implementation:validation>,
). La procédure `Validate` est invoquée par l'orchestrateur préalablement à
toute création ou mise à jour d'une spécification, afin d'obtenir du contrôleur
concerné la confirmation de sa validité ainsi que, le cas échéant, sa
spécification dérivée, tel qu'abordé dans le #chapter-full-ref(
    <ch:implementation:validation>,
). La procédure `Reconcile` est quant à elle invoquée afin de déclencher la
réconciliation de la ressource par le contrôleur qui en a la charge.

=== Sécurité et isolation
Le travail de semestre préconisait l'isolation de chaque composant au sein de
namespaces Linux distincts. Cette approche s'avère toutefois peu praticable dans
le cadre de l'implémentation retenue: les contrôleurs doivent notamment partager
un même namespace réseau ainsi qu'un même système de fichiers avec le reste du
système. En l'absence d'une isolation complète par namespace, la réduction de la
surface d'attaque de chaque composant repose sur la restriction de ses
privilèges au moyen des capabilities Linux @bib-man7-capns-caps. L'ensemble des
capabilities est retiré par défaut de chaque composant; seules celles
strictement nécessaires à l'exécution d'un contrôleur donné, telles que
`CAP_NET_ADMIN` pour le contrôleur réseau, lui sont accordées. Ces capabilities
ne sont en outre activées que durant l'exécution d'une réconciliation, et non de
manière permanente durant l'ensemble du cycle de vie du composant, ce qui réduit
la durée durant laquelle ces permissions sont exposées.

== Orchestrateur <ch:implementation:scheduler>
La mise à jour groupée d'un ensemble de ressources, invoquée par l'API lors de
la réception d'une nouvelle configuration, est prise en charge par la procédure
`bulk_upsert`. Cette procédure calcule d'abord, par comparaison entre l'ensemble
des ressources précédemment connues et l'ensemble des ressources nouvellement
soumises, trois catégories: les ressources ajoutées, absentes de l'état
précédent; les ressources supprimées, absentes de la nouvelle configuration; et
les ressources modifiées, présentes dans les deux ensembles, mais dont la
spécification diffère. Les ressources ajoutées et modifiées sont ensuite
validées, tel que décrit dans le #chapter-full-ref(
    <ch:implementation:validation>,
) puis enregistrées.

La procédure `schedule_available` détermine enfin, parmi les ressources ainsi
ajoutées ou modifiées, celles dont l'ensemble des dépendances est satisfait: une
dépendance est considérée satisfaite si la ressource correspondante est
elle-même dans un état prêt ou terminé. Seules les ressources dont l'ensemble
des dépendances est ainsi satisfait sont effectivement planifiées pour
réconciliation immédiate au sein de la file d'attente, les autres demeurant en
attente jusqu'à ce que leurs dépendances respectives atteignent un état
compatible.

=== File d'attente
Comme indiqué dans le #chapter-full-ref(<ch:system-design:scheduling>), la
boucle de réconciliation dépend d'une file d'attente, structure représentée ici
par `Queue<K>`. Cette file est implémentée de manière générique: elle permet de
planifier n'importe quel élément de type `K` à un moment précis. Dans le cadre
du présent système, `K` représente l'identifiant unique d'une ressource. Son
état interne est principalement inclus dans la structure `QueueInner<K>`
illustrée dans le #code-num-ref(<code-queue-inner>):

#figure(
    label: <code-queue-inner>,
    caption: [Structure interne de la file d'attente],
    source: link("cmd/state-manager/src/queue.rs"),
    ```rust
    struct QueueInner<K> {
        scheduled: HashMap<K, Instant>,
        queue: BTreeMap<Instant, HashSet<K>>,
    }
    ```,
)

Les ressources ainsi planifiées sont stockées dans un dictionnaire basé sur un
B#{ sym.hyph.nobreak }arbre (`BTreeMap`)~@bib-rust-std-btreemap, indexé par
l'instant auquel leur réconciliation est prévue. Le recours à un B-arbre
facilite la récupération des ressources dont la planification est arrivée à
échéance: la date de réconciliation étant une valeur numérique ordonnée, toutes
les valeurs inférieures à l'instant présent correspondent à des échéances
passées et peuvent être traitées. L'implémentation Rust des B-arbres permet en
outre de scinder l'arbre en deux à partir d'une clé donnée, ce qui correspond
précisément au cas d'utilisation recherché. L'arbre étant par ailleurs ordonné,
cela permet de parcourir les éléments en commençant par le plus ancien, sans
recourir à une étape de tri additionnelle.

La valeur associée à une clé n'est pas un identifiant unique, mais un ensemble
d'identifiants. En effet, lorsqu'une nouvelle configuration est soumise,
l'ensemble des ressources nouvellement définies doit être planifié simultanément
pour réconciliation; c'est pourquoi chaque clé temporelle est associée à un
ensemble d'identifiants (`HashSet`) @bib-rust-std-hashset plutôt qu'à un
identifiant unique. Un champ supplémentaire, `scheduled`, complète cette
structure en fournissant un index inversé permettant de déterminer rapidement
si, et à quel moment, une ressource donnée est planifiée.

Cette file expose la méthode asynchrone `drain_expired()`, qui attend qu'une ou
plusieurs clés arrivent à échéance avant de les retourner. L'attente est
passive: la fonction calcule le délai jusqu'à la prochaine échéance connue, puis
se met en pause jusqu'à ce délai. Pour gérer le cas où un élément serait
planifié pendant cette attente, avec une échéance plus proche que celle déjà
calculée, ou alors qu'aucune échéance n'existait, un canal de notification est
utilisé, basé sur la structure `Notify` du runtime asynchrone Tokio.

Étant donné que plusieurs opérations d'écriture (ajout individuel, ajout en
masse, replanification) peuvent modifier l'échéance la plus proche et donc
nécessiter une notification, ces opérations passent par un `QueueInnerGuard`
illustré dans le #code-num-ref(<code-queue-gard>):

#figure(
    label: <code-queue-gard>,
    caption: [Encapsulation de la file d'attente pour les opérations
        d'écriture],
    source: link("cmd/state-manager/src/queue.rs"),
    ```rust
    struct QueueInnerGuard<'a, K> {
        guard: RwLockWriteGuard<'a, QueueInner<K>>,
        notify: Arc<Notify>,
        earliest: Option<Instant>,
    }
    ```,
)

Au moment de sa construction, ce guard capture l'échéance la plus proche connue
avant modification. Lorsqu'il est détruit (`Drop` @bib-rust-std-drop,
l'équivalent Rust d'un `defer` en Go ou d'un destructeur en programmation
orientée objet), il compare cette échéance initiale à l'échéance courante et
déclenche une notification si celle-ci a changé, que ce soit parce qu'une
échéance plus proche a été introduite, ou parce que la file est passée d'un état
non vide à vide ou inversement. Cette approche permet de simplifier la gestion
de l'envoi des notifications. En rendant ce processus entièrement transparent du
point de vue du code modifiant la file d'attente.

Enfin, la file est encapsulée dans une structure `Queue`, qui fournit un accès
concurrent aux différentes méthodes, illustré dans le #code-num-ref(
    <code-queue>,
):

#figure(
    label: <code-queue>,
    caption: [Encapsulation de la file d'attente pour un accès concurrent],
    source: link("cmd/state-manager/src/queue.rs"),
    ```rust
    struct Queue<K> {
        queue: Mutex<QueueInner<K>>,
        notify: Arc<Notify>,
    }
    ```,
)

L'accès à `QueueInner` est protégé par un Mutex @bib-rust-std-mutex, chaque
opération exposée par `Queue` nécessitant de toute façon un accès exclusif à la
structure sous-jacente.

// TODO?: Plus de détails

=== Réconciliation <ch:implementation:reconciliation>
La réconciliation est une tâche de fond qui récupère, en boucle, l'ensemble des
identifiants arrivés à échéance via `Queue::drain_expired`, puis transmet une
requête au contrôleur responsable afin de réconcilier chaque ressource. Cette
requête reprend la structure de ressource complète, décrite dans le
#code-num-ref(<code-resource-def>):

#figure(
    label: <code-resource-def>,
    caption: [Structure d'une ressource transmise lors de la réconciliation],
    source: repo("rust/crates/cos-proto-reconciler/src/lib.rs"),
    ```rust
    pub struct Resource<Spec, DerivedSpec, State> {
        pub id: Identity,
        pub phase: Phase,
        pub status: Status,
        pub spec: Spec,
        pub derived_spec: DerivedSpec,
        pub state: Option<State>,
        pub children: Vec<TerminalResource<Value, Value, Value>>,
        pub dependencies: Vec<TerminalResource<Value, Value, Value>>,
        pub dependents: Vec<TerminalResource<Value, Value, Value>>,
    }
    ```,
)

Dans le #code-num-ref(<code-resource-def>), les champs `children`,
`dependencies` et `dependents` reposent sur le type `TerminalResource`, une
variante de `Resource` dans laquelle ces mêmes champs sont réduits à de simples
identifiants, ce qui évite la récursion de la structure, comme illustré dans le
#code-num-ref(<code-resource-term>):

#figure(
    label: <code-resource-term>,
    caption: [Structure d'une ressource terminale],
    source: repo("rust/crates/cos-proto-reconciler/src/lib.rs"),
    ```rust
    pub struct TerminalResource<Spec, DerivedSpec, State> {
        pub id: Identity,
        pub phase: Phase,
        pub status: Status,
        pub spec: Spec,
        pub derived_spec: DerivedSpec,
        pub state: Option<State>,
        pub children: HashSet<Identity>,
        pub dependencies: HashSet<Identity>,
        pub dependents: HashSet<Identity>,
    }
    ```,
)

Le message de réconciliation contient une ressource sous forme générique, le
type `Value` représentant un type générique susceptible d'être converti
ultérieurement en un type connu, ce mécanisme permettant à une ressource de
posséder des ressources enfants de types différents. Le contrôleur désérialise
d'abord cette ressource générique afin d'en identifier le schéma, puis
désérialise les champs génériques dans la structure finale correspondante. La
réconciliation d'une ressource est déclenchée toutes les 30 secondes, ou plus
tôt si le contrôleur notifie l'orchestrateur d'une nécessité de réconciliation
anticipée.

Les ressources arrivées à échéance sont réconciliées séquentiellement, dans
l'ordre de leur ancienneté, sans regroupement. Deux ressources consécutives
destinées au même contrôleur font ainsi l'objet de deux requêtes distinctes, la
seconde n'étant transmise qu'après réception de la réponse à la première. Un
échec affectant la réconciliation d'une ressource n'empêche pas la tentative de
réconciliation de la ressource suivante.

La réponse du contrôleur inclut le nouvel état, le statut, l'ensemble des
enfants, et l'ensemble des dépendances, dont la structure est décrite dans le
#code-num-ref(<code-response-def>):

#figure(
    label: <code-response-def>,
    caption: [Structure de réponse d'une réconciliation],
    source: repo("rust/crates/cos-proto-reconciler/src/lib.rs"),
    ```rust
    pub struct ResourceResponse<State> {
        pub status: Status,
        pub state: Option<State>,
        pub children: Vec<SubResource<Value>>,
        pub dependencies: HashSet<Identity>,
    }
    ```,
)

Le champ `status` indique le résultat de la réconciliation, tel que prêt, pas
prêt ou une erreur. Le champ `state`, optionnel, correspond au nouvel état de la
ressource lorsque la réconciliation en produit un; son absence signifie que
l'état de la ressource reste inchangé. Le champ `children` contient l'ensemble
des ressources enfants créées ou maintenues par le contrôleur à l'issue de cette
réconciliation. Le champ `dependencies` contient quant à lui l'ensemble des
ressources dont dépend la ressource réconciliée. Seul l'identifiant est transmis
pour les dépendances, tandis que l'identifiant et la spécification sont transmis
pour les enfants, ces derniers pouvant être nouvellement créés et donc inconnus
de l'orchestrateur.

Deux catégories d'erreur sont distinguées lors de la réconciliation: l'erreur de
protocole et l'erreur de logique. Une erreur de protocole survient lorsque les
données échangées via gRPC sont invalides ou non interprétables, ou lorsqu'une
erreur imprévue se produit au niveau du transport. Une erreur de logique, en
revanche, correspond à un échec géré par le contrôleur dans le cadre normal de
la réconciliation, et se traduit par un statut d'erreur dans le champ `status`
de la réponse. Dans ce dernier cas, la réconciliation est considérée comme
réussie du point de vue du protocole gRPC, indépendamment du contenu du statut.
En cas d'erreur de protocole, l'orchestrateur attribue lui-même à la ressource
un statut d'erreur qualifié de "transport"; dans tous les autres cas, le statut
attribué à la ressource correspond à celui fourni par le contrôleur dans sa
réponse.

=== Validation d'une ressource <ch:implementation:validation>
La validation d'une ressource s'effectue à chaque création ou mise à jour d'une
spécification. Lorsque plusieurs ressources sont ajoutées ou modifiées
simultanément, l'ensemble de ces ressources est validé en parallèle. Une erreur
affectant un seul élément de cet ensemble entraîne l'échec de la validation pour
la totalité des ressources concernées.

La validation d'une ressource s'effectue à travers la procédure `validate()`. La
requête contient d'abord la nouvelle spécification de la ressource, et si cette
ressource existe déjà, la spécification courante de celle-ci ainsi que son état.
La ressource actuelle est transmise, car certaines ressources, telles que la
ressource permettant d'installer le système sur le disque, sont totalement ou
partiellement immuables; leur modification doit alors passer par la recréation
d'une nouvelle ressource.

La ressource préexistante est optionnelle dans la requête de validation, son
absence correspondant au cas d'une première réconciliation, pour laquelle aucun
état antérieur n'existe encore. Par ailleurs, certains champs de la
spécification peuvent être dérivés intégralement à partir de la spécification
elle-même, sans intervention de l'utilisateur. Le contrôleur fournit alors, en
réponse à la validation, une spécification dite dérivée, calculée uniquement à
partir de la spécification soumise. Cette spécification dérivée n'est recalculée
que lorsque la spécification change; elle ne peut pas être modifiée lors d'une
réconciliation normale. Ce mécanisme dispense l'utilisateur de renseigner
manuellement des champs entièrement déductibles, tout en évitant un recalcul
systématique de ces champs à chaque réconciliation. La garantie que la
spécification dérivée existe à chaque réconciliation permet en outre au
contrôleur de traiter ce champ comme systématiquement disponible, ce qui allège
la logique de réconciliation en dispensant celle-ci de toute vérification de
présence des champs dérivés.

== Contrôleurs
Chaque contrôleur implémente le service gRPC `ReconcilerService` décrit
précédemment, en particulier les procédures `validate` et `reconcile`, selon une
approche commun. La réception d'une requête entraîne d'abord la désérialisation
de la ressource générique reçue, puis un branchement est effectué sur le schéma
de cette ressource, comme illustré dans le #code-num-ref(<code-dispatch>) pour
la procédure `reconcile` du contrôleur réseau:

#figure(
    label: <code-dispatch>,
    caption: [Aiguillage sur le schéma d'une ressource],
    source: link("rust/cmd/network-controller/src/main.rs"),
    ```rust
    async fn reconcile(
        &self,
        request: Request<ReconcileRequest>,
    ) -> Result<Response<ReconcileResponse>, Status> {
        let req = request.into_inner();
        let resource: Resource<Value, Value, Value> =
            serde_json::from_slice(&req.raw)
                .map_err(|err| Status::from_error(err.into()))?;

        match resource.id.key().schema.as_ref() {
            "network:link" => {
                reconcile!(resource, LinkResource, {
                    let (conn, handle, _) = new_connection()?;
                    tokio::spawn(conn);
                    LinkReconciler::new_with(handle)
                });
            }
            // ...
            _ => return Err(Status::not_found("schema does not exist")),
        }
    }
    ```,
)

Chaque branche désérialise la ressource générique dans la structure typée
correspondant au schéma identifié, instancie le sous-contrôleur (`Reconciler`)
responsable de ce type de ressource avec les paramètres adéquats, puis lui
délègue le traitement de la requête. La macro `reconcile!` permet de masquer une
grande partie de la logique liée à la désérialisation de la ressource.

La logique de réconciliation propre à chaque type de ressource suit un
déroulement identique, structuré en quatre étapes: la spécification de la
ressource est d'abord validée; l'état physique courant de la ressource est
ensuite récupéré (`refresh`); un plan d'action est calculé par comparaison entre
cet état et la spécification (`plan`); ce plan est enfin exécuté (`apply`),
avant qu'un second `refresh` ne détermine l'état résultant réel de la ressource.
Chacune de ces étapes est susceptible d'échouer indépendamment des autres; un
échec à n'importe quelle étape interrompt immédiatement la réconciliation et
retourne une réponse dont le champ `status` est mis à `Error`, tout en
conservant, autant que possible, le dernier état connu de la ressource.

Le statut final d'une réconciliation ayant abouti dépend du plan calculé lors de
la seconde évaluation et de l'état courant de la ressource: l'absence de toute
action à entreprendre (`NoOp`) alors que la ressource est en cours de
suppression indique qu'elle a été supprimée avec succès ou qu'elle n'a jamais
existé; en dehors de tout contexte de suppression, l'absence d'action correspond
au statut prêt. À l'inverse, la persistance d'un plan non vide à l'issue de son
exécution indique que, malgré les actions correctives entreprises, l'état réel
de la ressource ne correspond toujours pas à la spécification attendue, et
aboutit à un statut pas prêt. Certaines ressources peuvent également retourner
un statut terminé plutôt que prêt, lorsque des réconciliations ultérieures
n'auraient pas de sens, par exemple un conteneur dont la politique de
redémarrage est "never".

=== Contrôleur système
Le contrôleur système, implémenté dans #repo("rust/cmd/system-controller"),
prend actuellement en charge la seule gestion du contenu du répertoire `/etc/`
#footnote[
    Par exemple, les certificats des autorités de certification racine, tels que
    Cloudflare, SwissSign, etc.
], à travers la ressource `system:etc` #footnote[
    Implémentée dans #repo(
        "rust/cmd/system-controller/src/resources/etc-file.rs",
    ).
].

L'écriture d'un fichier au sein de `/etc/` est effectuée de manière atomique.
Lorsque la spécification de la ressource change, un nouveau fichier temporaire
est créé au moyen du flag `O_TMPFILE` @bib-linux-open-tmpfile: ce mécanisme
permet de créer un descripteur de fichier associé à un inode dépourvu de tout
lien dans l'arborescence, garantissant qu'aucun processus tiers ne puisse
observer ou accéder à ce fichier avant que son contenu ne soit intégralement
écrit. L'ensemble des opérations nécessaires (écriture du contenu, application
des permissions, etc.) est ensuite effectué sur ce fichier temporaire. Une fois
ces opérations terminées, le fichier est rendu visible dans l'arborescence au
moyen de `linkat`~@bib-linkat, qui associe l'inode déjà constitué à son chemin
final; cette opération remplace atomiquement le fichier existant, le cas
échéant, sans jamais exposer d'état intermédiaire.

Le chemin spécifié dans la spécification de la ressource doit désigner un
fichier réel, sans aucun lien symbolique, et ne comporter aucun élément relatif
permettant de sortir du répertoire attendu. Cette contrainte est imposée par
l'appel `openat2` @bib-linux-openat2, combiné aux flags de résolution
`RESOLVE_NO_SYMLINKS` et `RESOLVE_BENEATH`, qui interdisent respectivement la
traversée de tout lien symbolique rencontré lors de la résolution du chemin, et
toute sortie du répertoire racine désigné par ce même appel. Cette double
contrainte prévient toute exploitation, par une spécification malveillante ou
erronée, d'un lien symbolique ou d'une séquence relative telle que "`..`" afin
d'accéder à un fichier situé en dehors du répertoire visé. Cet appel retournant
un descripteur de fichier, celui-ci est ensuite réutilisé pour créer le fichier
temporaire dans ce même répertoire, sans nouvelle résolution de chemin, ce qui
évite toute race condition entre la validation et la création. Cela permet
d'appliquer le principe "Time of Check, Time of Use" (TOCTOU) @bib-toctou.

=== Contrôleur réseau
Le contrôleur réseau, implémenté dans #repo("rust/cmd/network-controller"), gère
l'ensemble des ressources du domaine réseau. L'ensemble de ces ressources
communique avec le noyau au moyen du protocole netlink @bib-linux-rtnetlink, via
la bibliothèque Rust rtnetlink @bib-rtnetlink. Les ressources `network:dns` et
`network:address` ne présentent aucune particularité notable: l'état physique
correspondant est d'abord récupéré, puis comparé à la spécification, avant
d'être créé s'il est absent.

La ressource `link` #footnote[
    Implémentée dans #repo("rust/cmd/network-controller/src/resources/link.rs").
] se distingue par le nombre élevé d'attributs qui la composent. De plus, l'API
exposée par la netlink ne retourne pas directement une structure représentant un
lien réseau, mais un message contenant une liste d'attributs hétérogènes, dont
seul un sous-ensemble est pertinent, et dont le sous-ensemble effectivement
présent varie selon le lien concerné, comme illustré dans le #code-num-ref(
    <code-link-attr>,
):

#figure(
    label: <code-link-attr>,
    caption: [Extrait d'une liste d'attributs retournée pour un lien réseau],
    source: link("rust/cmd/network-controller/src/resources/link.rs"),
    ```rust
    vec![
        LinkAttribute::Name(/* ... */),
        LinkAttribute::OperState(/* ... */),
        LinkAttribute::Index(/* ... */),
        // many more...
    ]
    ```,
)

Or, en Rust, l'instanciation d'une structure requiert de fournir une valeur pour
chacun de ses champs, ce qui empêche de construire directement la structure
finale, illustrée dans le #code-num-ref(<code-link-base-strict>), à partir d'une
liste d'attributs incomplète:

#figure(
    label: <code-link-base-strict>,
    caption: [Structure finale représentant l'état d'un lien réseau],
    source: link("rust/cmd/network-controller/src/resources/link.rs"),
    ```rust
    #[derive(Builder /*, ... */)]
    #[builder(pattern = "mutable", vis = "pub(crate)")]
    pub struct LinkState {
        pub index: u32,
        pub running: bool,
        pub admin_up: bool,
        pub oper_state: LinkOperState,
        // ...
    }
    ```,
)

Rendre l'ensemble des champs de cette structure optionnels ne constitue pas une
solution satisfaisante, la plupart de ces champs étant en réalité obligatoires
dans l'état final d'un lien réseau. Un pattern "builder" est employé pour
résoudre ce problème: la macro `Builder`, appliquée à `LinkState`, génère
automatiquement une structure intermédiaire, `LinkStateBuilder`, illustrée dans
le #code-num-ref(<code-link-builder-strict>), dans laquelle chaque champ est
rendu optionnel:

#figure(
    label: <code-link-builder-strict>,
    caption: [Structure intermédiaire générée par la macro `Builder`],
    source: link("rust/cmd/network-controller/src/resources/link.rs"),
    ```rust
    pub struct LinkStateBuilder {
        pub index: Option<u32>,
        pub running: Option<bool>,
        pub admin_up: Option<bool>,
        pub oper_state: Option<LinkOperState>,
        // ...
    }
    ```,
)

Cette structure intermédiaire évite au contrôleur de devoir gérer manuellement,
au moyen d'une boucle et de variables intermédiaires dédiées, l'accumulation
progressive des attributs reçus avant de pouvoir construire la structure finale.
Chaque attribut reconnu dans le message netlink renseigne directement le champ
correspondant de cette structure intermédiaire, comme illustré dans le
#code-num-ref(<code-link-builder>):

#figure(
    label: <code-link-builder>,
    caption: [Extrait du constructeur de la ressource `link`],
    source: link("rust/cmd/network-controller/src/resources/link.rs"),
    ```rust
    fn build_from_attrs(
        &mut self, // self: LinkStateBuilder
        attrs: &[LinkAttribute]
    ) -> Result<()> {
        // ...


        for attr in attrs {
            match attr {
                LinkAttribute::Mtu(mtu) => { self.mtu(*mtu); }
                LinkAttribute::OperState(s) => { self.oper_state((*s).into()); }
                // ...
                _ => {}
            }
        }


        // ...
    }
    ```,
)

Une fois l'ensemble des attributs du message traités, la structure intermédiaire
`LinkStateBuilder` est converti vers la structure finale `LinkState`, cette
conversion échouant si l'un des champs obligatoires de `LinkState` demeure non
spécifié à l'issue du traitement.

La ressource `network:route` présente une particularité: avec la librairie
rtnetlink, la récupération d'une route unique par son identifiant ne retourne
pas l'entrée correspondante, mais tente de résoudre la route associée à cet
identifiant, ce qui ne correspond pas au comportement recherché. Le contrôleur
récupère par conséquent l'ensemble des routes existantes, puis filtre ce
résultat afin d'isoler celle qui correspond à la ressource gérée.

La ressource `network:dhcp` repose sur la bibliothèque smoltcp @bib-smoltcp,
seule bibliothèque Rust fournissant un client DHCP réutilisable comme composant
logiciel indépendant. Le protocole DHCP étant par nature asynchrone et piloté
par des échéances propres au serveur DHCP plutôt que par le cycle de
réconciliation du contrôleur, la première réconciliation de cette ressource se
limite à démarrer une tâche d'arrière-plan chargée de piloter le client DHCP.
Lorsque cette tâche reçoit une nouvelle configuration du serveur, elle
l'enregistre, puis notifie l'orchestrateur. L'orchestrateur replanifie alors la
réconciliation de la ressource, à l'occasion de laquelle le contrôleur récupère
la configuration ainsi obtenue et crée les sous-ressources `network:address` et
`network:route` correspondantes.

=== Contrôleur de conteneurs
Le contrôleur de conteneurs, implémenté dans #repo(
    "rust/cmd/container-controller",
), s'appuie sur Podman comme runtime de conteneurs, et sur la bibliothèque
Bollard @bib-bollard pour communiquer avec celui-ci. Lors de la création d'un
runtime, un port lui est attribué de manière arbitraire, sur lequel Podman
expose son API, utilisée ensuite par le contrôleur pour l'ensemble des
opérations relatives aux ressources du domaine de la conteneurisation.
// TODO?: Plus de détails

== API et clients d'administration <ch:implementation:api>
L'API constitue l'unique surface d'administration du système et se repose aussi
sur gRPC. Les procédures disponibles correspondent à celles présentées dans le
#table-full-ref(<tab-cli-commands>) présenté dans le #chapter-num-ref(
    <ch:functional-overview>,
). Chaque procédure exposée par le service `ApiService` est précédée d'une
vérification d'authentification (`auth_or_fail`), qui interrompt la requête
avant tout traitement en cas d'échec.

La procédure `push_config` constitue le point d'entrée principal de l'API: elle
reçoit l'ensemble des ressources composant la configuration déclarative du
système tel que présenté dans le #chapter-full-ref(<ch:functional-overview>).
Cette procédure identifie, parmi ces ressources, celle de schéma `api`, dont la
présence est obligatoire, ainsi qu'une éventuelle ressource de schéma `install`.
La configuration de l'API elle-même est mise à jour, puis l'ensemble des
ressources soumises remplace, par une opération de mise à jour groupée
(`bulk_upsert`), les ressources précédemment connues du système; les ressources
absentes de la nouvelle configuration sont ainsi implicitement supprimées.
Lorsque le système se trouve en mode maintenance et qu'une ressource `install`
est présente, cette procédure déclenche en outre, une fois la mise à jour des
ressources effectuée, l'installation du système sur le disque, suivie d'un
redémarrage automatique après un court délai, processus décrit plus en détail
dans le #chapter-full-ref(<ch:implementation:install>).

== Système de fichier racine <ch:implementation:rootfs>
Le système repose sur un fonctionnement presque entièrement immuable. Le système
de fichiers racine contient le strict minimum, à savoir les binaires dans
`/bin/` et quelques fichiers statiques dans `/etc/`. Ces deux répertoires sont
fournis au système à travers une archive SquashFS @bib-squashfs qui les rend
totalement immuables. Cette archive représente le système de fichiers racine
(`/`). Toutefois, certains fichiers doivent pouvoir être écrits dans `/etc/`
durant le fonctionnement normal du système, par exemple pour configurer la
résolution DNS ou Podman. Afin de permettre cela, un système de fichiers
temporaire est superposé à l'archive SquashFS grâce à OverlayFS @bib-overlayfs.
Ce système de fichiers temporaire est entièrement persisté en mémoire; ainsi,
lorsque le système d'exploitation redémarre, l'ensemble de son contenu est
perdu. Ceci ne constitue pas un problème compte tenu du modèle déclaratif du
système: l'ensemble de ces fichiers est en réalité dérivé de la configuration,
et recréé identique à chaque redémarrage.

== Démarrage du système
Le bootloader, quel qu'il soit, charge le noyau ainsi que l'initrd en mémoire.
L'initrd est un système de fichiers racine minimal et temporaire, dont le seul
rôle est de charger le système de fichiers racine réel décrit au
#chapter-full-ref(<ch:implementation:rootfs>). Une fois le noyau démarré,
celui-ci exécute le processus `/init` se trouvant sur l'initrd; dans le cas du
présent système, ce processus est le seul présent sur ce système de fichiers. Ce
processus, implémenté dans #repo("rust/cmd/init"), a pour but de localiser
l'archive SquashFS du système de fichiers racine réel, où qu'elle se trouve, de
la préparer, puis d'y apposer la surcouche d'écriture au moyen d'OverlayFS.

Pour localiser le système de fichiers racine réel, l'`/init` se base sur le
paramètre de démarrage `cos.bootdisk`, qui spécifie le disque et le numéro de
partition sur lesquels se trouvent les artefacts de démarrage. L'`/init` monte
alors temporairement cette partition afin d'y récupérer l'archive SquashFS. Si
ce paramètre est absent, l'`/init` considère qu'il est démarré depuis l'image
ISO, et tente de monter cette dernière afin d'y récupérer l'archive. Dans les
deux cas, l'archive se nomme `root.squashfs` et se trouve à la racine respective
de son support.

Une fois le système de fichiers racine réel mis en place, l'`/init` passe la
main au superviseur, implémenté dans #repo("rust/cmd/supervisor"), responsable
de monter le reste du système de fichiers. Cette répartition des rôles entre
`/init` et le superviseur s'explique par le fait que l'initrd, contenant
l'`/init`, est chargé en mémoire et doit à ce titre demeurer aussi léger que
possible. Le superviseur suit ensuite les étapes illustrée dans la
#figure-num-ref(<sysinit>):

#include "../diagrams/sysinit.typ"

Le superviseur construit l'arborescence de fichiers standard de Linux telle que
spécifiée par le Filesystem Hierarchy Standard @bib-linux-fhs. Il met ensuite en
place le réseau local de l'hôte, l'interface `localhost` n'étant pas activée par
défaut sous Linux; cette interface est essentielle au fonctionnement du système,
les différents contrôleurs communiquant entre eux via une adresse locale. Puis
monte les différents volumes additionnels spécifiés dans les paramètres de
démarrage, tels que `/config/` via `cos.configdisk` ou `/var/` via
`cos.datadisk`. Le superviseur démarre alors ces contrôleurs et attend qu'ils
soient prêts avant de démarrer l'orchestrateur. À partir de ce point, le
superviseur n'a plus d'autre rôle que d'attendre l'extinction du système.

Si l'une de ces étapes échoue, pour quelque raison que ce soit, le programme
s'interrompt et déclenche un "kernel panic". À ce stade du cycle de vie du
système, les composants permettant une gestion déclarative ne sont pas encore
chargés et il n'y a donc aucune autre possibilité pour joindre le système.

La #figure-num-ref(<img:kernel-panic>) illustre un tel comportement, provoqué
par l'indisponibilité du disque de configuration attendu par le système:

#figure(
    label: <img:kernel-panic>,
    caption: [Panique du noyau lorsqu'un disque n'est pas disponible],
    note: [Le noyau tente de monter le disque de configuration, mais celui-ci
        n'est pas disponible alors qu'il devrait l'être.],
    source: made-by-self,
    image("../../lib/assets/kernel-panic.png"),
)

En l'absence du disque de configuration, le système ne dispose d'aucune
spécification à réconcilier et son exécution ne présenterait aucune utilité,
tout en risquant d'exposer le système à des risques inutiles. La
#figure-num-ref(<procstart>) illustre l'arbre des processus démarrés par le
système, ainsi que les relations de filiation entre ceux-ci:

#include "../diagrams/procstart.typ"

Le processus `/init` démarre le superviseur, responsable de lancer les autres
processus, en commençant par les contrôleurs puis l'orchestrateur. Les
différents contrôleurs peuvent, à leur tour, démarrer divers processus, tels
qu'un runtime de conteneur ou un client DHCP.

== Installation et chiffrement <ch:implementation:install>
L'installation du système sur disque est déclenchée par l'API, via la ressource
`install`, et prise en charge par une procédure d'installation qui ne s'exécute
que si le système est démarré en mode maintenance.

Lorsqu'une configuration comportant une ressource `install` est soumise à l'API,
l'ensemble des ressources qu'elle contient est d'abord validé, puis enregistré
dans l'état interne du système, tel que décrit au #chapter-full-ref(
    <ch:implementation:api>,
). La file d'attente de réconciliations est toutefois suspendue avant que cette
installation ne soit commencée, ce qui empêche toute réconciliation d'une
ressource et garantit que l'installation s'exécute sur un état stable.

Le disque système est partitionné selon un schéma composé de quatre partitions:
une partition de boot de 1 MiB, une partition contenant les fichiers de Limine
ainsi que l'archive SquashFS du système, le noyau et l'initrd (`limine`, 512
MiB), une partition optionnelle destinée au disque de configuration (`config`,
10 MiB), et une partition optionnelle de données occupant l'espace restant
(`data`). Limine est ensuite installé sur le disque, les partitions requises
sont formatées et montées, avant que les fichiers de démarrage extraits de
l'image ISO ne soient copiés vers la partition `limine`. Une configuration de
démarrage est enfin générée, définissant deux entrées: l'une destinée à
l'exécution normale du système, référençant les disques présents par leur chemin
respectif au moyen de paramètres de démarrage dédiés, l'autre destinée au mode
maintenance.

Le chiffrement des disques repose sur LUKS @bib-luks, combiné à Clevis
@bib-clevis pour l'automatisation du déchiffrement. Le mode de chiffrement
appliqué à chaque disque est spécifié indépendamment, via un paramètre de
démarrage dédié de la forme `cos.<disktype>.encryption`, où `<disktype>` désigne
l'un des disques gérés par le système, tel que `configdisk` ou `datadisk`.

== Système de build
L'environnement de build repose sur l'outil Nix. Une distinction est requise
entre trois usages du terme "Nix": Nix en tant que système de build
@bib-nix-build, Nix en tant que gestionnaire de paquets @bib-nixpkgs, et Nix en
tant que distribution Linux (NixOS) @bib-nixos. Seul le premier usage, complété
partiellement par le second, est utilisé dans le cadre de ce projet. Les
concepts essentiels de Nix nécessaires à la compréhension de ce chapitre sont
présentés à l'#appendix-num-ref(<appendix:nix-primer>).

Le recours à Nix vise à fournir un environnement stable entre la machine de
développement locale et l'environnement d'intégration continue. Nix permet non
seulement la construction des artefacts, mais aussi la mise en place d'un
environnement de développement ou l'exécution de commandes au sein d'un
environnement isolé. La reproductibilité bit à bit des builds, permise par Nix,
constitue une propriété désirée pour le projet. Nix facilite en outre la mise en
œuvre de la cross-compilation, soit la compilation depuis une architecture CPU
particulière vers une architecture différente, par exemple, de x86_64 vers
ARM64.

L'ensemble de la chaîne de build du système est pris en charge par Nix: le
noyau, les différentes crates composant le projet, ainsi que les artefacts
finaux assemblés à partir de ces éléments. Le détail de cet assemblage est
développé ultérieurement dans le #chapter-full-ref(
    <ch:implementation:system-image>,
).

La compilation s'effectue via le compilateur nightly de Rust. Ce choix est
motivé par le recours à la fonctionnalité `build-std`, permettant le build de la
bibliothèque standard avec les mêmes options d'optimisation que le reste du
projet (plutôt que d'utiliser une version pre-build), ce qui contribue à la
minimisation de la taille des artefacts finaux, ainsi que par l'utilisation de
règles de linting propres au canal nightly. Le qualificatif nightly, ou
"unstable", désigne l'absence de garantie de pérennité des fonctions, options et
autres éléments exposés par ce canal, ces derniers pouvant être modifiés ou
retirés sans préavis. Cette instabilité porte uniquement sur la stabilité de
l'interface exposée dans le temps, et non sur la fiabilité d'exécution de la
fonctionnalité elle-même @bib-rust-unstable.

== Configuration du noyau
La configuration du noyau repose sur la configuration par défaut propre à
l'architecture cible, complétée par un ensemble d'options supplémentaires
situées dans le fichier #repo("linux/config/common.conf"). Plusieurs fragments
de configuration peuvent être fournis simultanément; une fonction Nix est créée
à cet effet, permettant de fusionner l'ensemble de ces fragments avec la
configuration par défaut. Cette approche permet de ne stocker, dans le dépôt,
que les changements apportés à la configuration par défaut, plutôt que la
configuration complète.

Une commande est mise à disposition afin de faciliter la modification de cette
configuration. Cette commande fusionne l'ensemble des fragments, ouvre
l'interface `menuconfig` du noyau, puis calcule automatiquement, à la sortie de
cette interface, la différence entre la configuration modifiée et trois
références distinctes. La différence par rapport au `defconfig` est exposée sous
`config.merged`, la différence par rapport à la configuration personnalisée sous
`config.diff`, et la configuration complète sous `config.full`.

== Génération de l'image du système <ch:implementation:system-image>
L'image finale du système, qu'il s'agisse d'une image ISO ou d'une image disque
brute, est assemblée entièrement au moyen de Nix. Chaque crate Rust du projet
correspond à un output Nix; l'ensemble de ces outputs est regroupé dans un autre
output nommé `rootfsEnv`, lequel intègre également des binaires additionnels,
tels que Podman. Cet output génère un dossier regroupant l'ensemble de ces
éléments au sein du `/nix/store`.

Un output `rootfs` reprend cet environnement et y ajoute les liens symboliques
nécessaires, de sorte que les répertoires `/bin`, `/etc`, etc., contiennent des
liens symboliques pointant vers le `/nix/store`. Cet output produit, en sortie,
une archive au format SquashFS. Deux outputs additionnels complètent cet
assemblage: `kernel`, qui construit l'image du noyau selon les options de
configuration retenues, et initrd, qui fournit le système de fichiers initial.

À partir de ces trois outputs, `rootfs`, `initrd` et `kernel`, un output `iso`
assemble l'ensemble en une image ISO, exécutable via QEMU @bib-qemu ou sur un
système physique. L'arbre de build est illustré dans la #figure-num-ref(<img>):

#include "../diagrams/img.typ"

L'assemblage destiné à d'autres architectures suit une démarche similaire. Une
image disque brute est par ailleurs requise pour les systèmes ne pouvant
démarrer depuis une image ISO et nécessitant d'être flashés, tels que le
Raspberry Pi. Un output spécifique est créé pour chaque système visé par ce mode
de déploiement. Dans le cas du Raspberry Pi, l'output `rpi-sd-image` regroupe
les éléments propres à cette plateforme et produit une image directement
destinée au flashage sur une carte SD.

== Environnement de test <ch:implementation:tests>
Trois catégories de tests sont mises en œuvre: les tests unitaires, les tests
d'intégration, et les tests de bout en bout (end-to-end). Les tests unitaires
sont dépourvus de complexité particulière, n'interagissant pas ou peu avec
l'environnement extérieur. Ces tests sont réalisés à l'aide du système de test
standard de Rust, via la macro `#[test]`.

Les tests d'intégration présentent une complexité supérieure. Le test du
contrôleur réseau en constitue un exemple représentatif: une exécution directe
sur l'hôte est exclue, la configuration réseau de ce dernier étant affectée,
tandis qu'une exécution au sein du système complet introduirait de nombreux
points de défaillance supplémentaires, certains éléments internes n'étant par
ailleurs pas exposés à ce niveau. Une crate nommée `isolation`#footnote[
    Implémenté dans #repo("rust/crates/isolation")
] est créée à cet effet, permettant l'exécution d'un test dans un processus
séparé, via un appel `fork` @bib-man2-fork, isolé par des namespaces Linux
@bib-man-ns. Un environnement entièrement vierge est ainsi fourni pour chaque
test, dépourvu d'interface réseau et disposant d'un système de fichiers racine
propre; l'ensemble de cet environnement est détruit à l'issue du test. Cette
isolation complète empêche cependant le test de binaires externes non présents
dans l'environnement vierge ainsi constitué. Ces tests sont, comme les tests
unitaires, réalisés via la macro~`#[test]`, et localisés à proximité du code
testé.

Les tests unitaires et les tests d'intégration partagent la propriété de ne pas
nécessiter la reconstruction du système complet, ne reposant que sur
l'écosystème Rust. Ces deux catégories de tests peuvent ainsi être exécutées
directement au sein de l'environnement de développement Nix, la commande de test
standard utilisant la compilation incrémentale de Rust plutôt qu'un rebuild
complet.

Les tests de bout en bout exécutent le système complet ainsi qu'une suite
d'actions et de mises à jour de configuration. Ces tests sont regroupés dans une
crate dédiée, nommée `e2e`#footnote[
    Implémenté dans #repo("rust/e2e")
]. Cette catégorie de test vise à reproduire les conditions d'utilisation finale
du système, ce qui nécessite l'exécution de l'image ISO complète. L'exécution de
ces tests repose sur le lancement d'une machine virtuelle via QEMU, à partir de
cette image ISO. Cette dépendance à l'image complète empêche l'exécution directe
de ces tests via la commande de test standard de Rust; leur exécution passe par
le système de build dans son intégralité.

L'ensemble des tests Rust est exécuté via l'outil Nextest @bib-nextest. Ce
recours n'est pas strictement nécessaire pour les tests unitaires, mais s'avère
requis pour les tests d'intégration. L'isolation propre à ces derniers provoque,
en cas d'échec, un appel système `exit` @bib-exit qui interrompt immédiatement
le système de test standard de Rust, alors que Nextest poursuit son exécution,
chaque test étant exécuté dans un thread distinct dont la terminaison est gérée
indépendamment.

== Pipeline CI/CD
L'ensemble du projet reposant sur Nix, la pipeline CI/CD (#repo(
    ".gitlab-ci.yml",
)) en hérite également, ce qui simplifie sa mise en œuvre: les étapes exécutées
par la pipeline se limitent, dans une large mesure, à l'invocation de commandes
Nix génériques, indépendantes de l'environnement d'exécution. L'ordonnancement
des tâches repose sur GitLab CI @bib-gitlab-ci. Le recours à un runner
personnalisé est requis, les runners institutionnels ne permettant ni
l'utilisation de Nix ni la virtualisation imbriquée, requise pour l'exécution
des tests de bout en bout, et disposant par ailleurs de ressources de calcul
limitées.

La pipeline est déclenchée à chaque push sur le dépôt, entraînant l'exécution de
l'ensemble des étapes suivantes, dans l'ordre. Une phase de linting est exécutée
en premier lieu, portant à la fois sur la documentation et sur le code. Une
phase de test lui succède, structurée séquentiellement: les tests unitaires et
les doctests de Rust#footnote[
    Un doctest est un exemple de code intégré à la documentation d'une fonction
    ou d'un module, compilé et exécuté automatiquement lors de l'exécution des
    tests afin de garantir la validité des exemples fournis @bib-rust-doctest.
] sont exécutés en parallèle, suivis des tests d'intégration. La phase de build
est ensuite exécutée, suivie enfin des tests de bout en bout. L'ensemble du
processus est illustré dans la #figure-num-ref(<cicd>):

#include "../diagrams/cicd.typ"

Le linting couvre deux aspects distincts: le formatage et l'analyse statique. Le
formatage est vérifié via typstfmt @bib-typstfmt pour la documentation, buf
@bib-buf pour les définitions Protocol Buffers, et l'outil natif pour le code
Rust @bib-cargo-fmt. L'analyse statique repose aussi sur buf pour les
définitions Protocol Buffers et sur Clippy @bib-cargo-clippy pour le code Rust.
Les options les plus strictes de Clippy sont activées, interdisant notamment le
recours à `unwrap` ou à des méthodes équivalentes, ainsi que l'utilisation de
`println`, afin de garantir que toute sortie passe par le système de
journalisation. Diverses règles additionnelles sont par ailleurs activées afin
d'assurer l'homogénéité du code. Toute désactivation ponctuelle d'une règle doit
être accompagnée d'une justification explicite, mécanisme nativement supporté
par Clippy; une exception dépourvue de justification est rejetée par la
pipeline. Une exception générale est toutefois appliquée au code de test, pour
lequel le recours à `unwrap` est autorisé, cette approche constituant la méthode
recommandée pour exprimer une assertion dans ce contexte.

Afin de simplifier l'invocation des commandes fréquemment utilisées, telles que
le linting, Just @bib-justfile, alternative au Makefile @bib-makefile, est mis
en place afin de définir des alias aux commandes fréquemment utilisées, telles
que le linting ou l'exécution des tests.
