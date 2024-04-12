# Utiliser l'image officielle d'Ubuntu comme image de base
FROM ubuntu:latest

# Installer les dépendances nécessaires pour Apache et Rust
RUN apt-get update && \
    apt-get install -y apache2 curl build-essential && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Installer Rust via rustup (installeur officiel de Rust)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Ajouter cargo et rustc au PATH
ENV PATH="/root/.cargo/bin:${PATH}"

# Copier les fichiers du site dans le répertoire public d'Apache
COPY . /var/www/html

# Exposer le port 80
EXPOSE 80

# Lancer Apache en arrière-plan
CMD ["apachectl", "-D", "FOREGROUND"]
