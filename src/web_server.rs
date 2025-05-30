use crate::models::{GlobalState};
use std::sync::{Arc, Mutex};
use warp::Filter;

type SharedGlobalState = Arc<Mutex<GlobalState>>;

pub struct WebServer {
    global_state: SharedGlobalState,
    port: u16,
}

impl WebServer {
    pub fn new(global_state: SharedGlobalState, port: u16) -> Self {
        Self { global_state, port }
    }
    
    pub async fn start(self) {
        let state_filter = warp::any().map(move || Arc::clone(&self.global_state));
        
        let api_status = warp::path!("api" / "status")
            .and(warp::get())
            .and_then(get_status);
        
        let api_repositories = warp::path!("api" / "repositories")
            .and(warp::get())
            .and(state_filter.clone())
            .and_then(get_repositories);
        
        let api_repository = warp::path!("api" / "repository" / String)
            .and(warp::get())
            .and(state_filter.clone())
            .and_then(get_repository);
        
        let api_builds = warp::path!("api" / "builds")
            .and(warp::get())
            .and(state_filter.clone())
            .and_then(get_recent_builds);
        
        let api_build = warp::path!("api" / "build" / u64)
            .and(warp::get())
            .and(state_filter)
            .and_then(get_build_detail);
        
        let index = warp::path::end()
            .and(warp::get())
            .and_then(serve_index);
        
        let routes = index
            .or(api_status)
            .or(api_repositories)
            .or(api_repository)
            .or(api_builds)
            .or(api_build);

        println!("🌐 Turbulent CI web interface available at http://localhost:{}", self.port);
        
        warp::serve(routes)
            .run(([127, 0, 0, 1], self.port))
            .await;
    }
}

async fn get_status() -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::reply::json(&serde_json::json!({"status": "running"})))
}

async fn get_repositories(state: SharedGlobalState) -> Result<impl warp::Reply, warp::Rejection> {
    let state = state.lock().unwrap();
    let repositories: Vec<_> = state.repositories.values().collect();
    Ok(warp::reply::json(&repositories))
}

async fn get_repository(repo_name: String, state: SharedGlobalState) -> Result<impl warp::Reply, warp::Rejection> {
    let state = state.lock().unwrap();
    if let Some((_, repo_state)) = state.repositories.iter().find(|(_, rs)| rs.repository.name == repo_name) {
        Ok(warp::reply::json(repo_state))
    } else {
        Ok(warp::reply::json(&serde_json::json!({"error": "Repository not found"})))
    }
}

async fn get_recent_builds(state: SharedGlobalState) -> Result<impl warp::Reply, warp::Rejection> {
    let state = state.lock().unwrap();
    Ok(warp::reply::json(&state.recent_builds))
}

async fn get_build_detail(id: u64, state: SharedGlobalState) -> Result<impl warp::Reply, warp::Rejection> {
    let state = state.lock().unwrap();
    if let Some(build) = state.recent_builds.iter().find(|b| b.id == id) {
        Ok(warp::reply::json(build))
    } else {
        Ok(warp::reply::json(&serde_json::json!({"error": "Build not found"})))
    }
}

async fn serve_index() -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::reply::html(HTML_TEMPLATE))
}

const HTML_TEMPLATE: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Turbulent CI Multi-Repository Dashboard</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif; background: #f8fafc; }
        .container { max-width: 1400px; margin: 0 auto; padding: 20px; }
        .header { background: white; padding: 24px; border-radius: 12px; margin-bottom: 24px; box-shadow: 0 4px 6px rgba(0,0,0,0.07); border: 1px solid #e2e8f0; }
        .header h1 { font-size: 32px; margin-bottom: 8px; color: #1e293b; }
        .header .subtitle { color: #64748b; font-size: 16px; }

        .nav-tabs { display: flex; gap: 8px; margin-bottom: 24px; }
        .nav-tab { background: white; border: 1px solid #e2e8f0; padding: 12px 20px; border-radius: 8px; cursor: pointer; transition: all 0.2s; color: #64748b; font-weight: 500; }
        .nav-tab.active { background: #3b82f6; color: white; border-color: #3b82f6; }
        .nav-tab:hover:not(.active) { background: #f8fafc; }

        .tab-content { display: none; }
        .tab-content.active { display: block; }

        .repo-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(350px, 1fr)); gap: 20px; margin-bottom: 24px; }
        .repo-card { background: white; border-radius: 12px; padding: 20px; box-shadow: 0 4px 6px rgba(0,0,0,0.07); border: 1px solid #e2e8f0; }
        .repo-header { display: flex; justify-content: between; align-items: center; margin-bottom: 16px; }
        .repo-name { font-size: 18px; font-weight: 600; color: #1e293b; }
        .repo-status { display: flex; align-items: center; gap: 8px; }

        .status { display: inline-block; padding: 6px 12px; border-radius: 20px; font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; }
        .status.passing { background: #dcfce7; color: #166534; }
        .status.failed { background: #fecaca; color: #991b1b; }
        .status.building { background: #fef3c7; color: #92400e; }
        .status.idle { background: #e2e8f0; color: #475569; }
        .status.error { background: #fee2e2; color: #991b1b; }

        .project-type { background: #e0e7ff; color: #3730a3; padding: 4px 8px; border-radius: 12px; font-size: 10px; font-weight: 600; }

        .repo-info { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin-bottom: 16px; }
        .repo-detail { }
        .repo-detail strong { display: block; color: #475569; font-size: 12px; text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 4px; }
        .repo-detail div { color: #1e293b; font-weight: 500; }

        .recent-builds { margin-top: 16px; }
        .recent-builds h4 { color: #475569; font-size: 14px; margin-bottom: 12px; }
        .build-item-small { display: flex; align-items: center; gap: 8px; padding: 8px 0; border-bottom: 1px solid #f1f5f9; }
        .build-item-small:last-child { border-bottom: none; }
        .build-icon { font-size: 14px; }
        .build-info-small { flex: 1; }
        .build-id { font-weight: 600; color: #1e293b; font-size: 13px; }
        .build-time { font-size: 11px; color: #64748b; }

        .builds-section { background: white; border-radius: 12px; box-shadow: 0 4px 6px rgba(0,0,0,0.07); border: 1px solid #e2e8f0; }
        .builds-header { padding: 20px; border-bottom: 1px solid #f1f5f9; display: flex; justify-content: between; align-items: center; }
        .builds-header h2 { color: #1e293b; }
        .filter-buttons { display: flex; gap: 8px; }
        .filter-btn { padding: 6px 12px; border: 1px solid #e2e8f0; background: white; border-radius: 6px; font-size: 12px; cursor: pointer; }
        .filter-btn.active { background: #3b82f6; color: white; border-color: #3b82f6; }

        .build-item { padding: 20px; border-bottom: 1px solid #f1f5f9; display: flex; align-items: center; justify-content: space-between; transition: background-color 0.2s; }
        .build-item:hover { background: #f8fafc; }
        .build-item:last-child { border-bottom: none; }
        .build-info { flex: 1; }
        .build-header { display: flex; align-items: center; gap: 12px; margin-bottom: 8px; }
        .build-meta { font-size: 13px; color: #64748b; display: flex; gap: 16px; flex-wrap: wrap; }
        .build-actions { display: flex; gap: 8px; }
        .btn { padding: 8px 16px; border: none; border-radius: 8px; cursor: pointer; font-size: 12px; font-weight: 500; transition: all 0.2s; }
        .btn-primary { background: #3b82f6; color: white; }
        .btn-primary:hover { background: #2563eb; }
        .btn-secondary { background: #f1f5f9; color: #475569; }
        .btn-secondary:hover { background: #e2e8f0; }

        .modal { display: none; position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); z-index: 1000; }
        .modal-content { background: white; margin: 2% auto; padding: 24px; width: 95%; max-width: 900px; border-radius: 12px; max-height: 90vh; overflow-y: auto; }
        .output { background: #0f172a; color: #e2e8f0; padding: 20px; border-radius: 8px; font-family: 'SF Mono', Monaco, 'Cascadia Code', monospace; font-size: 13px; white-space: pre-wrap; line-height: 1.5; }
        .refresh-btn { position: fixed; bottom: 24px; right: 24px; background: #3b82f6; color: white; border: none; padding: 16px; border-radius: 50%; cursor: pointer; box-shadow: 0 8px 25px rgba(59, 130, 246, 0.3); font-size: 18px; }
        .empty-state { padding: 60px 20px; text-align: center; color: #64748b; }
        .loading { padding: 40px; text-align: center; color: #94a3b8; }

        .repo-path { font-family: 'SF Mono', Monaco, monospace; font-size: 12px; background: #f8fafc; padding: 4px 8px; border-radius: 4px; color: #475569; }

        .summary-stats { display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 16px; margin-bottom: 24px; }
        .stat-card { background: white; padding: 16px; border-radius: 8px; border: 1px solid #e2e8f0; text-align: center; }
        .stat-number { font-size: 24px; font-weight: 700; color: #1e293b; }
        .stat-label { font-size: 12px; color: #64748b; text-transform: uppercase; letter-spacing: 0.5px; margin-top: 4px; }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🌪️ Turbulent CI Dashboard</h1>
            <div class="subtitle">Multi-Repository Continuous Integration</div>
        </div>

        <div class="nav-tabs">
            <div class="nav-tab active" onclick="switchTab('overview')">📊 Overview</div>
            <div class="nav-tab" onclick="switchTab('repositories')">📁 Repositories</div>
            <div class="nav-tab" onclick="switchTab('builds')">🔨 Recent Builds</div>
        </div>

        <div id="overview-tab" class="tab-content active">
            <div class="summary-stats" id="summary-stats">
                <div class="loading">Loading statistics...</div>
            </div>
            <div class="repo-grid" id="repo-overview">
                <div class="loading">Loading repositories...</div>
            </div>
        </div>

        <div id="repositories-tab" class="tab-content">
            <div class="repo-grid" id="repositories-container">
                <div class="loading">Loading repositories...</div>
            </div>
        </div>

        <div id="builds-tab" class="tab-content">
            <div class="builds-section">
                <div class="builds-header">
                    <h2>Recent Builds</h2>
                    <div class="filter-buttons">
                        <button class="filter-btn active" onclick="filterBuilds('all')">All</button>
                        <button class="filter-btn" onclick="filterBuilds('passing')">Passing</button>
                        <button class="filter-btn" onclick="filterBuilds('failed')">Failed</button>
                    </div>
                </div>
                <div id="builds-container">
                    <div class="loading">Loading builds...</div>
                </div>
            </div>
        </div>

        <button class="refresh-btn" onclick="loadAllData()" title="Refresh">🔄</button>
    </div>

    <div id="build-modal" class="modal">
        <div class="modal-content">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 24px;">
                <h2 style="color: #1e293b;">Build Details</h2>
                <button onclick="closeModal()" style="background: none; border: none; font-size: 24px; cursor: pointer; color: #64748b;">&times;</button>
            </div>
            <div id="build-details"></div>
        </div>
    </div>

    <script>
        let repositories = [];
        let recentBuilds = [];
        let currentFilter = 'all';

        async function loadAllData() {
            await Promise.all([
                loadRepositories(),
                loadRecentBuilds()
            ]);
            renderCurrentTab();
        }

        async function loadRepositories() {
            try {
                const response = await fetch('/api/repositories');
                repositories = await response.json();
            } catch (error) {
                console.error('Failed to load repositories:', error);
                repositories = [];
            }
        }

        async function loadRecentBuilds() {
            try {
                const response = await fetch('/api/builds');
                recentBuilds = await response.json();
            } catch (error) {
                console.error('Failed to load builds:', error);
                recentBuilds = [];
            }
        }

        function switchTab(tabName) {
            // Update nav tabs
            document.querySelectorAll('.nav-tab').forEach(tab => tab.classList.remove('active'));
            event.target.classList.add('active');

            // Update tab content
            document.querySelectorAll('.tab-content').forEach(content => content.classList.remove('active'));
            document.getElementById(tabName + '-tab').classList.add('active');

            renderCurrentTab();
        }

        function renderCurrentTab() {
            const activeTab = document.querySelector('.tab-content.active');
            if (activeTab.id === 'overview-tab') {
                renderOverview();
            } else if (activeTab.id === 'repositories-tab') {
                renderRepositories();
            } else if (activeTab.id === 'builds-tab') {
                renderBuilds();
            }
        }

        function renderOverview() {
            renderSummaryStats();
            renderRepositoryOverview();
        }

        function renderSummaryStats() {
            const container = document.getElementById('summary-stats');

            const totalRepos = repositories.length;
            const passingRepos = repositories.filter(r => r.current_status === 'Passing').length;
            const failingRepos = repositories.filter(r => r.current_status === 'Failed').length;
            const totalBuilds = recentBuilds.length;
            const successRate = totalBuilds > 0 ? Math.round((recentBuilds.filter(b => b.success).length / totalBuilds) * 100) : 0;

            container.innerHTML = `
                <div class="stat-card">
                    <div class="stat-number">${totalRepos}</div>
                    <div class="stat-label">Repositories</div>
                </div>
                <div class="stat-card">
                    <div class="stat-number" style="color: #059669;">${passingRepos}</div>
                    <div class="stat-label">Passing</div>
                </div>
                <div class="stat-card">
                    <div class="stat-number" style="color: #dc2626;">${failingRepos}</div>
                    <div class="stat-label">Failing</div>
                </div>
                <div class="stat-card">
                    <div class="stat-number" style="color: #3b82f6;">${totalBuilds}</div>
                    <div class="stat-label">Total Builds</div>
                </div>
                <div class="stat-card">
                    <div class="stat-number" style="color: ${successRate >= 80 ? '#059669' : successRate >= 60 ? '#d97706' : '#dc2626'};">${successRate}%</div>
                    <div class="stat-label">Success Rate</div>
                </div>
            `;
        }

        function renderRepositoryOverview() {
            const container = document.getElementById('repo-overview');

            if (repositories.length === 0) {
                container.innerHTML = '<div class="empty-state">🌪️ No repositories configured<br><small>Use CLI to add repositories: <code>turbulent-ci add ./path/to/repo</code></small></div>';
                return;
            }

            container.innerHTML = repositories.map(repo => {
                const recentBuilds = repo.builds.slice(0, 3);
                return `
                    <div class="repo-card">
                        <div class="repo-header">
                            <div>
                                <div class="repo-name">${repo.repository.name}</div>
                                <div class="repo-path">${repo.repository.path}</div>
                            </div>
                            <div class="repo-status">
                                <span class="project-type">${repo.repository.project_type}</span>
                                <span class="status ${repo.current_status.toLowerCase()}">${repo.current_status}</span>
                            </div>
                        </div>

                        <div class="repo-info">
                            <div class="repo-detail">
                                <strong>Branch</strong>
                                <div>${repo.repo_info.branch}</div>
                            </div>
                            <div class="repo-detail">
                                <strong>Last Commit</strong>
                                <div>${repo.repo_info.last_commit.substring(0, 8)}</div>
                            </div>
                        </div>

                        <div class="recent-builds">
                            <h4>Recent Builds</h4>
                            ${recentBuilds.length > 0 ? recentBuilds.map(build => `
                                <div class="build-item-small">
                                    <span class="build-icon">${build.success ? '✅' : '❌'}</span>
                                    <div class="build-info-small">
                                        <div class="build-id">Build #${build.id}</div>
                                        <div class="build-time">${new Date(build.timestamp * 1000).toLocaleString()}</div>
                                    </div>
                                    <button class="btn btn-secondary" onclick="showBuildDetails(${build.id})">View</button>
                                </div>
                            `).join('') : '<div style="color: #64748b; font-size: 12px;">No builds yet</div>'}
                        </div>
                    </div>
                `;
            }).join('');
        }

        function renderRepositories() {
            const container = document.getElementById('repositories-container');

            if (repositories.length === 0) {
                container.innerHTML = '<div class="empty-state">🌪️ No repositories configured<br><small>Use CLI to add repositories: <code>turbulent-ci add ./path/to/repo</code></small></div>';
                return;
            }

            container.innerHTML = repositories.map(repo => `
                <div class="repo-card">
                    <div class="repo-header">
                        <div>
                            <div class="repo-name">${repo.repository.name}</div>
                            <div class="repo-path">${repo.repository.path}</div>
                        </div>
                        <div class="repo-status">
                            <span class="project-type">${repo.repository.project_type}</span>
                            <span class="status ${repo.current_status.toLowerCase()}">${repo.current_status}</span>
                        </div>
                    </div>

                    <div class="repo-info">
                        <div class="repo-detail">
                            <strong>Branch</strong>
                            <div>${repo.repo_info.branch}</div>
                        </div>
                        <div class="repo-detail">
                            <strong>Last Commit</strong>
                            <div>${repo.repo_info.last_commit}</div>
                        </div>
                        <div class="repo-detail">
                            <strong>Total Builds</strong>
                            <div>${repo.builds.length}</div>
                        </div>
                        <div class="repo-detail">
                            <strong>Success Rate</strong>
                            <div>${repo.builds.length > 0 ? Math.round((repo.builds.filter(b => b.success).length / repo.builds.length) * 100) : 0}%</div>
                        </div>
                    </div>

                    <div style="margin-top: 16px;">
                        <strong style="color: #475569; font-size: 12px; text-transform: uppercase;">Commands:</strong>
                        <div style="margin-top: 8px; font-family: 'SF Mono', Monaco, monospace; font-size: 12px; background: #f8fafc; padding: 12px; border-radius: 6px; border: 1px solid #e2e8f0;">
                            ${repo.repository.commands.map(cmd => `<div>• ${cmd}</div>`).join('')}
                        </div>
                    </div>
                </div>
            `).join('');
        }

        function renderBuilds() {
            const container = document.getElementById('builds-container');

            let filteredBuilds = recentBuilds;
            if (currentFilter === 'passing') {
                filteredBuilds = recentBuilds.filter(b => b.success);
            } else if (currentFilter === 'failed') {
                filteredBuilds = recentBuilds.filter(b => !b.success);
            }

            if (filteredBuilds.length === 0) {
                container.innerHTML = '<div class="empty-state">🌪️ No builds found</div>';
                return;
            }

            container.innerHTML = filteredBuilds.map(build => `
                <div class="build-item">
                    <div class="build-info">
                        <div class="build-header">
                            <span style="font-size: 18px;">${build.success ? '✅' : '❌'}</span>
                            <strong style="font-size: 16px;">Build #${build.id}</strong>
                            <span class="status ${build.success ? 'passing' : 'failed'}">${build.success ? 'Passed' : 'Failed'}</span>
                            <span style="background: #f1f5f9; color: #475569; padding: 4px 8px; border-radius: 12px; font-size: 11px; font-weight: 600;">${build.repository_name}</span>
                        </div>
                        <div class="build-meta">
                            <span>📋 ${build.commit_hash.substring(0, 8)}</span>
                            <span>🕐 ${new Date(build.timestamp * 1000).toLocaleString()}</span>
                            <span>⏱️ ${build.duration_ms}ms</span>
                            <span>📁 ${build.repo_path}</span>
                        </div>
                    </div>
                    <div class="build-actions">
                        <button class="btn btn-primary" onclick="showBuildDetails(${build.id})">View Details</button>
                    </div>
                </div>
            `).join('');
        }

        function filterBuilds(filter) {
            currentFilter = filter;
            document.querySelectorAll('.filter-btn').forEach(btn => btn.classList.remove('active'));
            event.target.classList.add('active');
            renderBuilds();
        }

        async function showBuildDetails(buildId) {
            try {
                const response = await fetch(`/api/build/${buildId}`);
                const build = await response.json();

                if (build.error) {
                    alert('Build not found');
                    return;
                }

                const details = document.getElementById('build-details');
                details.innerHTML = `
                    <div style="margin-bottom: 24px;">
                        <h3 style="color: #1e293b; margin-bottom: 16px;">Build #${build.id} ${build.success ? '✅' : '❌'}</h3>
                        <div class="repo-info">
                            <div class="repo-detail">
                                <strong>Repository</strong>
                                <div>${build.repository_name}</div>
                            </div>
                            <div class="repo-detail">
                                <strong>Commit</strong>
                                <div>${build.commit_hash}</div>
                            </div>
                            <div class="repo-detail">
                                <strong>Path</strong>
                                <div>${build.repo_path}</div>
                            </div>
                            <div class="repo-detail">
                                <strong>Started</strong>
                                <div>${new Date(build.timestamp * 1000).toLocaleString()}</div>
                            </div>
                            <div class="repo-detail">
                                <strong>Duration</strong>
                                <div>${build.duration_ms}ms</div>
                            </div>
                            <div class="repo-detail">
                                <strong>Project Type</strong>
                                <div><span class="project-type">${build.project_type}</span></div>
                            </div>
                        </div>
                    </div>
                    <h4 style="color: #1e293b; margin-bottom: 12px;">Build Output:</h4>
                    <div class="output">${build.output || 'No output available'}</div>
                `;

                document.getElementById('build-modal').style.display = 'block';
            } catch (error) {
                console.error('Failed to load build details:', error);
            }
        }

        function closeModal() {
            document.getElementById('build-modal').style.display = 'none';
        }

        // Close modal when clicking outside
        window.onclick = function(event) {
            const modal = document.getElementById('build-modal');
            if (event.target === modal) {
                closeModal();
            }
        }

        // Auto-refresh every 15 seconds
        setInterval(loadAllData, 15000);

        // Initial load
        loadAllData();
    </script>
</body>
</html>
"#;
