#pragma once

#include <reactphysics3d/reactphysics3d.h>
#include <entt/entt.hpp>
#include <soloud.h>
#include <soloud_wav.h>
#include <vector>
#include <imgui.h>

#include "RenderObjects.hpp"
#include "InputProcessor.hpp"
#include "Renderer.hpp"

enum class GameState
{
    MainMenu,
    Game
};

using AudioResourceID = uint32_t;

class Engine
{
public:
    Engine();
    ~Engine();

    void init();
    AudioResourceID load_audio_resource(const char* path);
    void update(float delta_time, const InputProcessor& input_processor, bool& running, SDL_Window* window);
    void render_scene(float delta_time, const glm::vec2& resolution);
    void shutdown();
    SoLoud::Soloud& get_soloud() { return m_soloud; }
    SoLoud::AudioSource* get_audio_resources(AudioResourceID id) { return m_audio_resources[id]; }
private:
    reactphysics3d::PhysicsCommon m_physics_common;
    reactphysics3d::PhysicsWorld* m_physics_world;
    entt::registry m_registry;
    entt::entity m_player_entity{};
    entt::entity m_terrain_entity{};
    
    SoLoud::Soloud m_soloud;
    std::vector<SoLoud::Wav*> m_audio_resources;
    AudioResourceID m_audio_res_id_counter;

    Renderer m_renderer;

    RenderObjects<100> m_render_objects{};

    GameState m_game_state = GameState::Game;
};
