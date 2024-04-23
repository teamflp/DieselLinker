FROM ubuntu:23.10

# Mettre à jour et installer les dépendances nécessaires
RUN apt-get update && \
    apt-get install -y curl build-essential && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Installer Rust via rustup (installeur officiel de Rust)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Ajouter cargo et rustc au PATH
ENV PATH="/root/.cargo/bin:${PATH}"

# Copier les fichiers du projet Rust dans le conteneur
COPY . /usr/src/diesel_linker

# Changer le répertoire de travail pour le projet Rust
WORKDIR /usr/src/diesel_linker

# Commande par défaut pour garder le conteneur actif
CMD ["sleep", "infinity"]
