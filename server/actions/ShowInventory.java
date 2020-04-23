package server.actions;

import java.util.List;

import server.main.World;
import server.objects.Player;
import server.objects.Player.ReadOnlyPlayer;

public class ShowInventory implements Action {
    @Override
    public boolean matchCommand(String command) {
        return command.equals("inv");
    }

    @Override
    public boolean parseCommand(String command, ReadOnlyPlayer player, List<ReadOnlyPlayer> players, World world,
            StringBuilder error) {
        return true;
    }

    @Override
    public StringBuilder run(Player player, List<Player> players, World world) {
        return new StringBuilder(player.getInventory().toString());
    }
}