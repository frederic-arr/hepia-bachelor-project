# OS pour le déploiement de services conteneurisés

<p align="center">
  <b>Documents</b>
  <br />
  <a href="https://gitedu.hesge.ch/flg_bachelors/tb/2026/container-infrastructure-deployment-os/-/jobs/artifacts/main/raw/output/Enonce_LOG_diplome_Arroyo_Gluck_2026.pdf?job=build-docs-internal">Énoncé du sujet</a>&nbsp;&bull;&nbsp;
  <a href="https://gitedu.hesge.ch/flg_bachelors/tb/2026/container-infrastructure-deployment-os/-/jobs/artifacts/main/raw/output/ISC_LOG_resume_diplome_Arroyo_Gluck_2026.pdf?job=build-docs-internal">Résumé</a>&nbsp;&bull;&nbsp;
  <a href="https://gitedu.hesge.ch/flg_bachelors/tb/2026/container-infrastructure-deployment-os/-/jobs/artifacts/main/raw/output/ISC_LOG_memoire_diplome_Arroyo_Gluck_2026.pdf?job=build-docs-internal">Mémoire</a>&nbsp;&bull;&nbsp;
  <a href="https://gitedu.hesge.ch/flg_bachelors/tb/2026/container-infrastructure-deployment-os/-/jobs/artifacts/main/raw/output/slides/ISC_LOG_slides_handout-16-9_Arroyo_Gluck_2026.pdf?job=build-docs-internal">Slides</a>
  <br/>
  <small>Ce travail fait suite au <a href="https://gitedu.hesge.ch/flg_bachelors/ps/2025/container_os/-/raw/bachelor/final/arroyo-frederic-sp-2026.pdf?ref_type=tags&inline=true">projet de semestre</a></small>
</p>

ContainerOS est une distribution Linux indépendante conçue spécifiquement pour
le déploiement automatisé d'infrastructure de conteneurs. L'ensemble des
opérations, de l'installation à l'administration courrante, s'effectue
uniquement via une API, sans shell ou console interactive.

## Démarrage rapide

Cet exemple montre comment deployer ContainerOS et serveur HTTP sur une machine
virtuelle.

### Prérequis
1. Un environement Linux ou WSL
2. QEMU
3. Télécharger les fichiers de ContainerOS sur Switch Drive: https://drive.switch.ch/index.php/s/4N8DNLwJE45mth5

### Étapes
1. Sur **votre** machine, crée un fichier `containeros.yaml` avec le contenu
   suivant:
    ```yaml
    ---
    # Spécification de la méthode d'installation.
    # Ici toutes les partitions sont installée sur le disque /dev/vda
    schema: install
    disks:
      - dev: /dev/vda
        partitions:
          # boot contient le bootloader ainsi que les programmes nécessaire au
          # fonctionnement de l'OS.
          - type: boot

          # config contient la configuration courrante de l'OS. En son absencem,
          # la configuration est temporaire, c'est à dire qu'à chaque
          # redémarrage, le système sera "comme neuf".
          - type: config

          # data permet de persister les données. En son absence, toutes les
          # données sont effacée à  chaque redémarrage du système.
          - type: data

    ---
    # Permet de configurer l'accès à l'API. Dans le cadre de cet exemple,
    # l'authentification est désactivée étant donné qu'il s'agit d'un
    # environement local
    schema: api
    auth: none

    ---
    # L'exemple utilise une image se situant sur `docker.io` ce qui nécessite
    # une résolution de nom. ContainerOS donne le contrôle total et explicite de
    # tous les paramètres à l'administrateur.
    schema: network:dns
    nameservers:
      - 9.9.9.9

    ---
    # Comme dans toute distribution Linux, les interfaces réseau doivent être
    # activées. Ici aussi, le choix est explicite.
    schema: network:link
    name: eth0
    admin_up: true

    ---
    # QEMU fournit un serveur DHCP aux VM, le reste de la configuration de 
    # l'interface réseau se fera automatiquement via ce protocol.
    schema: network:dhcp
    name: eth0

    ---
    # ContainerOS utilise Podman pour exécuter des conteneurs. Il est possible
    # de créer plusieurs instances de Podman et d'y attribuer une utilisateur
    # différent. Ici, une instance "rootless" est créée. Le nom est arbitraire,
    # ce sont les paramètres uid/gid qui controlent l'aspect root/non-root.
    schema: container:runtime
    name: rootless
    engine: podman
    uid: 1000
    gid: 1000

    # Étant donné qu'il est nécessaire de faire une résolution DNS, il est
    # nécessaire de spécifier que le DNS *doit* être prêt avant. Idem pour le
    # réseau.
    depends_on:
      - network:dns
      - network:route/eth0-dhcp

    ---
    # Le conteneur est créé
    schema: container:instance
    name: demo
    image: docker.io/library/nginx:latest
    runtime: rootless
    ports:
      - container_port: 8080
        host_port: 8080
    ```
2. Crée un disque virtuel de 1 GiB:
   ```sh
   $ qemu-img create disk.img 1G
   ```
3. Démarrer la VM:
   ```sh
   $ qemu-system-x86_64 -cdrom containeros.iso \
      -drive file=disk.img,format=raw,if=virtio \
      -enable-kvm \
      -cpu host -m 256M \
      -netdev user,id=net0,hostfwd=tcp::50000-:50000,hostfwd=tcp::8080-:8080 \
      -device e1000,netdev=net0 \
      -nographic
   ```
   Les paramètres `hostfwd` permettent d'accèder à l'API et au conteneur de la VM.
4. Lorsque la VM est prête, appliquer la config:
   ```sh
   cosc --server http://127.0.0.1:50000 config push ../containeros.yaml
   ```
5. Après une minute, le conteneur devrait être disponible avec `curl http://127.0.0.1:8080` et afficher `Hello, world!`
6. Pour quitter la VM, appuier sur `CTRL+A` puis `C`, et entrer `q` pour couper la VM

## Mirrors

The source repository is on HEPIA's GitLab instance ([`flg_bachelors/tb/2026/container-infrastructure-deployment-os`](https://gitedu.hesge.ch/flg_bachelors/tb/2026/container-infrastructure-deployment-os))

This repository is mirrored on:
- Codeberg ([`frederic-arr/hepia-bachelor-project`](https://codeberg.org/frederic-arr/hepia-bachelor-project))
- GitHub ([`frederic-arr/hepia-bachelor-project`](https://github.com/frederic-arr/hepia-bachelor-project))
- A second namespace on HEPIA's GitLab instance ([`frederic.arroyo/bachelor-project`](https://gitedu.hesge.ch/frederic.arroyo/bachelor-project))
