package server.actions;

import java.util.List;

import server.main.World;
import server.objects.Player;
import server.objects.Player.ReadOnlyPlayer;

import server.utils.DisplayUtils;

public class ShowMap implements Action {

    public static final int CHUNK_SIZE = 30;

    @Override
    public boolean matchCommand(String command) {
        return command.equals("map");
    }

    @Override
    public boolean parseCommand(String command, ReadOnlyPlayer player, List<ReadOnlyPlayer> players, World world,
            StringBuilder error) {
        return true;
    }

    @Override
    public StringBuilder run(Player player, List<Player> players, World world) {
        StringBuilder out = new StringBuilder("");
        out.append(player.getPosn().toString() + "\n");
        out.append(DisplayUtils.map(CHUNK_SIZE, players, world));
        return out;
    }

    @Override
    public String description() {
        return "displays the world map.\n" +
               "usage: map\n";
    }
}