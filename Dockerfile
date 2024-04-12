# Utiliser l'image officielle d'Ubuntu comme image de base
FROM ubuntu:latest

# Installer Apache
RUN apt-get update && \
    apt-get install -y apache2 && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Copier les fichiers du site dans le répertoire public d'Apache
COPY . /var/www/html

# Exposer le port 80
EXPOSE 80

# Lancer Apache en arrière-plan
CMD ["apachectl", "-D", "FOREGROUND"]
