mod actions;
mod descriptions;
mod entity;
mod mcp;
mod persistence;
mod world;

use anyhow::Result;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use tiny_http::{Method, Request, Response, Server};
use tracing_subscriber::EnvFilter;

fn main() -> Result<()> {
    // Initialize logging to stderr (so it doesn't interfere with MCP protocol on stdout)
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_writer(std::io::stderr)
        .with_target(false)
        .init();

    tracing::info!("Rubber Duck MCP Server v{}", env!("CARGO_PKG_VERSION"));
    tracing::info!("A text-based healing nature simulation");

    // Determine state file path
    let state_path = get_state_path();
    tracing::info!("State file: {:?}", state_path);
    let log_path = get_log_path(&state_path);

    // Ensure data directory exists
    if let Some(parent) = state_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    start_web_server(state_path.clone(), log_path.clone());

    // Create and run the MCP server
    let mut server = mcp::McpServer::new(state_path, log_path);
    server.run()?;

    Ok(())
}

fn get_state_path() -> PathBuf {
    // Check for RUBBER_DUCK_STATE environment variable
    if let Ok(path) = std::env::var("RUBBER_DUCK_STATE") {
        return PathBuf::from(path);
    }

    // Default to current directory
    let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    path.push("data");
    path.push("world_state.json");
    path
}

fn get_log_path(state_path: &PathBuf) -> PathBuf {
    let mut path = state_path.clone();
    path.set_file_name("web_log.txt");
    path
}

fn start_web_server(state_path: PathBuf, log_path: PathBuf) {
    thread::spawn(move || {
        let mut port = 8080;
        let server = loop {
            match Server::http(("0.0.0.0", port)) {
                Ok(s) => {
                    tracing::info!("Web view available at http://localhost:{}", port);
                    break s;
                }
                Err(_) => {
                    port += 1;
                    if port > 8100 {
                        tracing::warn!("Unable to bind web server on ports 8080-8100");
                        return;
                    }
                }
            }
        };

        let map = world::WorldMap::new();
        loop {
            match server.recv_timeout(Duration::from_millis(250)) {
                Ok(Some(request)) => {
                    handle_http_request(request, &state_path, &log_path, &map);
                }
                Ok(None) => continue,
                Err(e) => {
                    tracing::warn!("Web server stopped: {}", e);
                    break;
                }
            }
        }
    });
}

fn handle_http_request(
    rq: Request,
    state_path: &PathBuf,
    log_path: &PathBuf,
    map: &world::WorldMap,
) {
    let url = rq.url().to_string();
    let method = rq.method().clone();
    match (method, url.as_str()) {
        (Method::Get, "/") => {
            let body = build_index_html();
            let _ = rq.respond(
                Response::from_string(body).with_header(
                    tiny_http::Header::from_bytes(
                        &b"Content-Type"[..],
                        &b"text/html; charset=utf-8"[..],
                    )
                    .unwrap(),
                ),
            );
        }
        (Method::Get, "/state") => {
            let body = build_state_json(state_path, map);
            let _ = rq.respond(
                Response::from_string(body).with_header(
                    tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
                        .unwrap(),
                ),
            );
        }
        (Method::Get, "/log") => {
            let body = build_log_json(log_path);
            let _ = rq.respond(
                Response::from_string(body).with_header(
                    tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
                        .unwrap(),
                ),
            );
        }
        _ => {
            let _ = rq.respond(Response::from_string("Not Found").with_status_code(404));
        }
    }
}

fn build_index_html() -> String {
    r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8" />
<title>Rubber Duck World</title>
<style>
body { margin:0; font-family: 'IBM Plex Mono', 'Fira Code', monospace; background:#0b0d11; color:#dce3ec; }
.wrap { display:flex; height:100vh; }
#map { flex:1; padding:12px 16px; box-sizing:border-box; background:#05070a; overflow:auto; }
pre { margin:0; font-size:14px; line-height:16px; }
.panel { width:50%; max-width:640px; padding:16px; box-sizing:border-box; overflow-y:auto; background:#0f131a; border-left:1px solid #1c2432; }
.logline { margin:0 0 8px 0; padding:8px; background:#141b26; border-radius:6px; border:1px solid #1f2935; }
.badge { display:inline-block; padding:2px 6px; margin-right:6px; border-radius:4px; font-size:12px; background:#233149; color:#9cc3ff; }
.legend { margin:0 0 8px 0; }
</style>
</head>
<body>
<div class="wrap">
  <div id="map"><pre id="map-pre"></pre></div>
  <div class="panel">
    <h2>Activity</h2>
    <div id="log"></div>
  </div>
</div>
<script>
const palette = {
  Desert:'#f2d16b',
  Oasis:'#49c3a9',
  SpringForest:'#7de07d',
  WinterForest:'#9dc7ff',
  Lake:'#5aa3ff',
  MixedForest:'#6ec06e',
  Path:'#d2a676',
   Clearing:'#e0d9c7',
  Cabin:'#ffd166',
  WoodShed:'#f48fb1',
  Player:'#ffda5a'
};

async function fetchJson(url) {
  const res = await fetch(url, {cache:'no-store'});
  if (!res.ok) throw new Error('fetch fail');
  return res.json();
}

function renderMap(data) {
  const pre = document.getElementById('map-pre');
  const lines = [];
  for (let r=0; r<data.height; r++) {
    let line = '';
    for (let c=0; c<data.width; c++) {
      const tile = data.tiles[r][c];
      const isPlayer = data.player && data.player.row === r && data.player.col === c;
      const visited = tile.visited !== false;
      const glyph = (() => {
        if (isPlayer) return '@';
        if (!visited) return '?';
        switch (tile.tile) {
          case 'Cabin': return 'C';
          case 'WoodShed': return 'W';
          case 'Clearing': return '.';
          case 'Path': return '#';
          case 'Lake': return '~';
          case 'CaveEntrance': return '>';
          case 'Forest': return tile.biome === 'WinterForest' ? '^' : tile.biome === 'Desert' ? '.' : 'T';
          default: return '.';
        }
      })();
      const color = isPlayer
        ? palette.Player
        : visited
          ? (palette[tile.biome] || '#9ea7b8')
          : '#3a4353';
      line += `<span style="color:${color}">${glyph}</span>`;
    }
    lines.push(line);
  }
  pre.innerHTML = lines.join('<br>');
}

function renderLog(lines) {
  const logEl = document.getElementById('log');
  logEl.innerHTML = '';
  lines.slice(-50).reverse().forEach(line => {
    const div = document.createElement('div');
    div.className = 'logline';
    div.innerHTML = `<span class="badge">log</span>${line}`;
    logEl.appendChild(div);
  });
}

async function tick() {
  try {
    const [state, log] = await Promise.all([fetchJson('/state'), fetchJson('/log')]);
    renderMap(state);
    renderLog(log);
  } catch (e) {
    console.error(e);
  } finally {
    setTimeout(tick, 1500);
  }
}
tick();
</script>
</body>
</html>
"#.to_string()
}

#[derive(serde::Serialize)]
struct StateView {
    width: usize,
    height: usize,
    player: Option<PositionView>,
    tiles: Vec<Vec<TileView>>,
}

#[derive(serde::Serialize)]
struct PositionView {
    row: usize,
    col: usize,
}

#[derive(serde::Serialize)]
struct TileView {
    biome: String,
    tile: String,
    visited: bool,
}

fn build_state_json(state_path: &PathBuf, map: &world::WorldMap) -> String {
    let loaded_state = persistence::GameState::load(state_path).ok();
    let object_view = loaded_state.as_ref().map(|s| &s.objects);
    let visited_view = loaded_state.as_ref().map(|s| &s.player.visited);
    let player_world_pos = loaded_state.as_ref().map(|s| s.player.position);

    let mut tiles = Vec::with_capacity(world::map::MAP_HEIGHT);
    for r in 0..world::map::MAP_HEIGHT {
        let mut row = Vec::with_capacity(world::map::MAP_WIDTH);
        for c in 0..world::map::MAP_WIDTH {
            if let Some(t) = map.get_tile(r, c) {
                let world_pos = world::Position::new(
                    r as i32 - world::map::MAP_ORIGIN_ROW,
                    c as i32 - world::map::MAP_ORIGIN_COL,
                );

                let mut tile = match t.tile_type {
                    world::TileType::Lake => "Lake",
                    world::TileType::Path => "Path",
                    world::TileType::Clearing => "Clearing",
                    world::TileType::Forest(_) => "Forest",
                }
                .to_string();

                if let Some(objects) = &object_view {
                    if objects
                        .objects_at(&world_pos)
                        .iter()
                        .any(|o| matches!(o.object.kind, world::ObjectKind::Cabin(_)))
                    {
                        tile = "Cabin".to_string();
                    } else if objects
                        .objects_at(&world_pos)
                        .iter()
                        .any(|o| matches!(o.object.kind, world::ObjectKind::WoodShed(_)))
                    {
                        tile = "WoodShed".to_string();
                    } else if objects
                        .objects_at(&world_pos)
                        .iter()
                        .any(|o| o.id == "east_cave_entrance"
                            || matches!(&o.object.kind, world::ObjectKind::GenericStructure(name) if name.to_lowercase().contains("cave")))
                    {
                        tile = "CaveEntrance".to_string();
                    }
                }

                let visited = match visited_view {
                    Some(set) => set.contains(&world_pos),
                    None => true,
                } || player_world_pos.map(|p| p == world_pos).unwrap_or(false);

                row.push(TileView {
                    biome: t.biome.name().to_string(),
                    tile,
                    visited,
                });
            }
        }
        tiles.push(row);
    }

    let mut player_pos = None;
    if let Some(state) = loaded_state.as_ref() {
        if let Some((row, col)) = state.player.position.as_usize() {
            player_pos = Some(PositionView { row, col });
        }
    }

    serde_json::to_string(&StateView {
        width: world::map::MAP_WIDTH,
        height: world::map::MAP_HEIGHT,
        player: player_pos,
        tiles,
    })
    .unwrap_or_else(|_| "{}".to_string())
}

fn build_log_json(log_path: &PathBuf) -> String {
    use std::fs;
    if let Ok(data) = fs::read_to_string(log_path) {
        let mut lines: Vec<_> = data.lines().map(|s| s.to_string()).collect();
        if lines.len() > 100 {
            lines = lines.split_off(lines.len().saturating_sub(100));
        }
        serde_json::to_string(&lines).unwrap_or_else(|_| "[]".to_string())
    } else {
        "[]".to_string()
    }
}
