use crate::models::PolicyGuide;

pub fn get_policy_guides() -> Vec<PolicyGuide> {
    vec![
        PolicyGuide {
            id: "jp_article_175_standard".to_string(),
            name: "Japanese Obscenity Standard (Penal Code Art. 175 & Commercial Doujin/Pixiv Baseline)".to_string(),
            legal_or_platform_basis: "Japanese Penal Code Article 175 (Distribution of Obscene Objects), Eirin / Computer Software Ethics Organization (EOCS) / Pixiv / DLsite industry standards.".to_string(),
            mosaic_rule: "Mosaic block size must be at least 1/100th of the longest image dimension (max(W, H) / 100), and never smaller than 4x4 pixels at the target publication resolution. After downscaling to 1080p or thumbnail sizes and JPEG compression, internal anatomical lines (urethral/vaginal cleft, glans corona, internal mucosal detail, explicit penetration contact points) must remain completely indistinguishable.".to_string(),
            min_mosaic_block_divisor: 100,
            absolute_min_mosaic_px: 4,
            artistic_nudity_guidance: "Do NOT automatically censor every instance of nudity. Artistic nudity, non-explicit silhouettes, classical/stylized breasts or buttocks without explicit genital depiction, and suggestive postures pass without intervention under this profile. Preserve the artist's work untouched unless explicit genitalia or sexual acts are depicted.".to_string(),
            explicit_criteria: vec![
                "Exposed male or female external genitalia (penis, vulva/labia, anus when explicitly spread/sexualized)".to_string(),
                "Explicit sexual acts, penetration, or genital contact points".to_string(),
                "Internal anatomical lines or contours of genitalia visible through sheer material or fluids".to_string(),
            ],
            escalation_triggers: vec![
                "Ambiguous stylized geometry where it is unclear whether forms represent explicit anatomy or abstract/biomechanical art".to_string(),
                "Censorship of the required area would obscure >25% of the focal composition or destroy the core artistic statement".to_string(),
                "Extreme/borderline fetish or violent themes where destination platform compliance cannot be resolved by localized mosaic/bars alone".to_string(),
            ],
        },
        PolicyGuide {
            id: "kuro_nuri_graphic".to_string(),
            name: "Japanese Graphic Bar Censorship (Kuro-nuri / Shiro-nuri Print Convention)".to_string(),
            legal_or_platform_basis: "Japanese commercial manga magazine & artbook convention using deliberate thin solid black or white bars over explicit clefts/anatomy.".to_string(),
            mosaic_rule: "Uses high-contrast solid bars (#000000 or #FFFFFF) precisely angled over explicit anatomical clefts or contact points, optionally paired with localized mosaic where bars alone would be excessively wide.".to_string(),
            min_mosaic_block_divisor: 100,
            absolute_min_mosaic_px: 4,
            artistic_nudity_guidance: "Ideal for high-contrast line art, monochrome ink, or graphic illustration where pixel mosaic disrupts aesthetic texture. Keep bars minimal, surgical, and aligned with the anatomical axis.".to_string(),
            explicit_criteria: vec![
                "Genital clefts, urethral/vaginal openings, glans details, and explicit contact points".to_string(),
            ],
            escalation_triggers: vec![
                "Cases where solid bars conflict with dark/light atmospheric lighting and require artistic consultation on bar vs. mosaic".to_string(),
            ],
        },
        PolicyGuide {
            id: "strict_social_safe".to_string(),
            name: "Mainstream Social / SFW Teaser Distribution Profile".to_string(),
            legal_or_platform_basis: "General social media teaser guidelines (X/Twitter sensitive-media threshold, Instagram/mainstream promotional previews).".to_string(),
            mosaic_rule: "Covers explicit genitalia as well as exposed female nipples/areolae and explicit pubic regions using localized mosaic, Gaussian blur, or graphic bars.".to_string(),
            min_mosaic_block_divisor: 80,
            absolute_min_mosaic_px: 6,
            artistic_nudity_guidance: "Used only when exporting promotional previews for strict platforms; preserves overall composition without cropping or redrawing.".to_string(),
            explicit_criteria: vec![
                "All explicit genitalia and sexual acts".to_string(),
                "Exposed female nipples/areolae".to_string(),
                "Explicitly exposed gluteal cleft or pubic hair in sexualized framing".to_string(),
            ],
            escalation_triggers: vec![
                "Full-body nude compositions where censoring required regions would obscure over 30% of the figure".to_string(),
            ],
        },
    ]
}

pub fn find_policy(id: &str) -> PolicyGuide {
    get_policy_guides()
        .into_iter()
        .find(|p| p.id == id)
        .unwrap_or_else(|| get_policy_guides().into_iter().next().unwrap())
}

/// Computes the recommended Japanese Article 175 compliant mosaic block size for an image of `(width, height)`.
pub fn recommended_mosaic_block_size(width: u32, height: u32, policy_id: &str) -> u32 {
    let policy = find_policy(policy_id);
    let max_dim = width.max(height);
    let computed = (max_dim as f32 / policy.min_mosaic_block_divisor as f32).ceil() as u32;
    computed.max(policy.absolute_min_mosaic_px).max(4)
}

