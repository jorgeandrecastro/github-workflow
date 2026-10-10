use colored::*;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::env;

#[derive(Deserialize, Debug)]
struct Repository {
    name: String,
    html_url: String,
}

#[derive(Deserialize, Debug)]
struct Branch {
    name: String,
}

fn main() {
    // 1. Récupérer le token GitHub depuis les variables d'environnement
    let token = match env::var("GITHUB_TOKEN") {
        Ok(t) => t,
        Err(_) => {
            eprintln!(
                "{}",
                "Erreur : La variable d'environnement GITHUB_TOKEN n'est pas définie.".red().bold()
            );
            eprintln!("Définissez-la avec : export GITHUB_TOKEN='votre_token'");
            std::process::exit(1);
        }
    };

    println!("\n{}", "========================================".cyan());
    println!("  PROJET : {} ", "github-workflow (GitHub API)".green().bold());
    println!("{}", "========================================".cyan());

    // 2. Initialiser le client HTTP avec les en-têtes requis par l'API GitHub
    let client = Client::new();
    let url = "https://api.github.com/user/repos?per_page=10";

    println!("Interrogation de l'API GitHub en cours...\n");

    let response = client
        .get(url)
        .header("User-Agent", "github-workflow-rust")
        .header("Authorization", format!("Bearer {}", token))
        .send();

    match response {
        Ok(res) => {
            if res.status().is_success() {
                let repos: Vec<Repository> = res.json().unwrap_or_default();
                
                if repos.is_empty() {
                    println!("Aucun dépôt trouvé sur votre compte.");
                } else {
                    println!("{}", "Vos dépôts GitHub récents :".underline());
                    for repo in repos {
                        println!("  📦 {} ({})", repo.name.green().bold(), repo.html_url.dimmed());
                        
                        // Optionnel : Récupérer les branches pour chaque dépôt
                        fetch_branches(&client, &token, &repo.name);
                    }
                }
            } else {
                eprintln!("Erreur API GitHub : Status {}", res.status());
            }
        }
        Err(e) => {
            eprintln!("Erreur réseau : {}", e);
        }
    }
    println!();
}

fn fetch_branches(client: &Client, token: &str, repo_name: &str) {
    // Récupérer le nom d'utilisateur authentifié pour l'URL des branches
    // (Simplifié ici, ou en passant par l'endpoint /user)
    let url = format!("https://api.github.com/repos/drew0670/{}/branches", repo_name);

    let res = client
        .get(&url)
        .header("User-Agent", "github-workflow-rust")
        .header("Authorization", format!("Bearer {}", token))
        .send();

    if let Ok(response) = res {
        if response.status().is_success() {
            let branches: Vec<Branch> = response.json().unwrap_or_default();
            for branch in branches {
                print!("    ㄴ 🌿 branch: {} ", branch.name.yellow());
            }
            println!();
        }
    }
}