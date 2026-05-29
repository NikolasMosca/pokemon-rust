use game_core::types::Type;

/// Descrive l'animazione di attacco da mostrare sullo sprite.
#[derive(Clone, PartialEq, Debug)]
pub enum AttackAnim {
    /// Colpo fisico: lo sprite scatta in avanti (lunge).
    Physical,
    /// Mossa speciale: charge + sfera colorata verso il bersaglio.
    Special { color: &'static str },
    /// Mossa di guarigione/drain: glow verde.
    Heal,
    /// Sprite colpito: trema lateralmente.
    Hit,
}

/// Restituisce la classe CSS del tipo da applicare alle card Pokémon.
pub fn type_css_class(t: &Type) -> &'static str {
    match t {
        Type::Fire     => "type-fire",
        Type::Water    => "type-water",
        Type::Electric => "type-electric",
        Type::Grass    => "type-grass",
        Type::Ice      => "type-ice",
        Type::Psychic  => "type-psychic",
        Type::Ghost    => "type-ghost",
        Type::Dark     => "type-dark",
        Type::Poison   => "type-poison",
        Type::Ground   => "type-ground",
        Type::Rock     => "type-rock",
        Type::Bug      => "type-bug",
        Type::Dragon   => "type-dragon",
        Type::Steel    => "type-steel",
        Type::Fairy    => "type-fairy",
        Type::Fighting => "type-fighting",
        Type::Flying   => "type-flying",
        Type::Normal   => "type-normal",
    }
}

/// Restituisce il colore CSS della sfera per una Special basata sul tipo.
pub fn orb_color(t: &Type) -> &'static str {
    match t {
        Type::Fire     => "#ff5020",
        Type::Water    => "#3080ff",
        Type::Electric => "#ffe020",
        Type::Grass    => "#30c030",
        Type::Ice      => "#80e8ff",
        Type::Psychic  => "#ff50c0",
        Type::Ghost    => "#7030c0",
        Type::Dark     => "#503070",
        Type::Poison   => "#a030c0",
        Type::Ground   => "#c09030",
        Type::Rock     => "#a08040",
        Type::Bug      => "#80a020",
        Type::Dragon   => "#4040e0",
        Type::Steel    => "#8090a0",
        Type::Fairy    => "#ff80c0",
        Type::Fighting => "#c03020",
        Type::Flying   => "#70a0e0",
        Type::Normal   => "#d0d0d0",
    }
}
