package server.actions;

import java.util.List;

import server.main.World;
import server.objects.Player;
import server.objects.Player.ReadOnlyPlayer;

public class ShowStats implements Action {

    @Override
    public boolean matchCommand(String command, int playerID) {
        return command.equals("stat");
    }

    @Override
    public boolean parseCommand(String command, ReadOnlyPlayer player, List<ReadOnlyPlayer> players, World world,
            StringBuilder error) {
        return true;
    }

    @Override
    public StringBuilder run(Player player, List<Player> players, World world) {
        StringBuilder out = new StringBuilder("");
        out.append("stats: \n");
        out.append(player.getStats().toString());
        out.append("base stats: \n");
        out.append(player.getBaseStats().toString());
        return out;
    }

    @Override
    public String description() {
        return "display your stats.\n" +
               "usage: stat";
    }
    
}