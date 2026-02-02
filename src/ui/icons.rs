
pub const DICE_SVG: &str = r#"<?xml version='1.0' encoding='UTF-8'?>
    <svg width='100' height='100' viewBox='0 0 100 100' xmlns='http://www.w3.org/2000/svg'>
        <rect x='5' y='5' width='90' height='90' rx='15' ry='15' fill='#ffffff' stroke='#333333' stroke-width='4'/>
        <g fill='#333333'>
        <circle cx='28' cy='28' r='8'/> <circle cx='72' cy='28' r='8'/> <circle cx='50' cy='50' r='8'/> <circle cx='28' cy='72' r='8'/> <circle cx='72' cy='72' r='8'/> </g>
    </svg>"#;

pub const WALLHAVEN_SVG: &str = r#"<?xml version='1.0' encoding='UTF-8'?>
<svg width='100' height='100' viewBox='0 0 100 100' xmlns='http://www.w3.org/2000/svg'>
  <rect x='5' y='5' width='90' height='90' rx='10' fill='#333333'/>

  <path d='M20 35 L28 65 L36 45 L44 65 L52 35'
        fill='none'
        stroke='#FFFFFF'
        stroke-width='8'
        stroke-linecap='round'
        stroke-linejoin='round'/>

  <path d='M62 35 L62 65 M80 35 L80 65 M62 50 L80 50'
        fill='none'
        stroke='#FFFFFF'
        stroke-width='8'
        stroke-linecap='round'
        stroke-linejoin='round'/>
</svg>"#;