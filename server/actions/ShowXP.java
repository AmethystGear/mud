package server.actions;

import java.util.List;

import server.main.World;
import server.objects.Player;
import server.objects.Player.ReadOnlyPlayer;

public class ShowXP implements Action {

    @Override
    public boolean matchCommand(String command) {
        return command.equals("xp");
    }

    @Override
    public boolean parseCommand(String command, ReadOnlyPlayer player, List<ReadOnlyPlayer> players, World world,
            StringBuilder error) {
        return true;
    }

    @Override
    public StringBuilder run(Player player, List<Player> players, World world) {
        return new StringBuilder("xp: " + player.xp().get());
    }

    @Override
    public String description() {
        return "displays your current xp.\n" +
               "usage: xp\n";
    }

}