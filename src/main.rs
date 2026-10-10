use colored::*;
use git2::{Repository, Status};

fn main() {
    // 1. Ouvrir le dépôt Git dans le répertoire courant
    let repo = match Repository::discover(".") {
        Ok(r) => r,
        Err(_) => {
            eprintln!("{}", "Erreur : Aucun dépôt Git trouvé ici.".red().bold());
            std::process::exit(1);
        }
    };

    // 2. Récupérer la branche actuelle
    let branch_name = match repo.head() {
        Ok(head) => {
            if head.is_branch() {
                head.shorthand().unwrap_or("inconnue").to_string()
            } else {
                "HEAD détache (detached HEAD)".to_string()
            }
        }
        Err(_) => "Aucun commit / HEAD vide".to_string(),
    };

    // 3. Affichage de l'en-tête visuel
    println!("\n{}", "========================================".cyan());
    println!("  PROJET : {} ", "github-workflow".green().bold());
    println!("  BRANCHE : {}", branch_name.yellow().bold());
    println!("{}", "========================================".cyan());

    // 4. Récupérer le statut des fichiers
    let statuses = match repo.statuses(None) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Erreur lors de la lecture du statut Git : {}", e);
            std::process::exit(1);
        }
    };

    if statuses.is_empty() {
        println!("\n✨ {} Tout est propre, aucun fichier modifié.", "[PROPRE]".green().bold());
    } else {
        println!("\n{}", "Fichiers modifiés / en cours :".underline());
        
        for entry in statuses.iter() {
            let path = entry.path().unwrap_or("chemin inconnu");
            let status = entry.status();

            let (label, color_status) = parse_git_status(status);
            println!("  [{}] {}", label.color(color_status), path);
        }
    }
    println!();
}

// Fonction utilitaire pour traduire les flags Git en labels lisibles et couleurs
fn fn_status_label(status: Status) -> (&'static str, &'static str) {
    if status.is_index_new() || status.is_index_modified() || status.is_index_deleted() {
        return ("STAGED", "green");
    }
    if status.is_wt_modified() {
        return ("MODIFIÉ", "yellow");
    }
    if status.is_wt_new() {
        return ("UNTRACKED", "blue");
    }
    if status.is_wt_deleted() {
        return ("SUPPRIMÉ", "red");
    }
    ("MODIFIÉ", "magenta")
}

// Wrapper pour correspondre à l'appel
fn parse_git_status(status: Status) -> (&'static str, &'static str) {
    fn_status_label(status)
}