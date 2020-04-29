package server.actions;

import java.util.List;
import java.util.Scanner;

import server.main.World;
import server.objects.Player;
import server.objects.Player.ReadOnlyPlayer;

public class Upgrade implements Action {
    private String stat;

    @Override
    public boolean matchCommand(String command, int playerID) {
        return command.startsWith("upgrade");
    }

    @Override
    public boolean parseCommand(String command, ReadOnlyPlayer player, List<ReadOnlyPlayer> players, World world,
            StringBuilder error) {
        Scanner scan = new Scanner(command);
        scan.next();
        if(!scan.hasNext()) {
            error.append("you need to type the stat to upgrade!");
            scan.close();
            return false;
        }
        stat = scan.next();
        scan.close();
        if(!player.getBaseStats().hasVariable(stat)) {
            error.append("you don't have that stat!");
            return false;
        }
        int xp = player.xp().get();
        int statLevel = (Integer)player.getStats().get(stat);
        int xpToLevel = statLevel * Player.XP_MULTIPLIER;
        if(xp < xpToLevel) {
            error.append("you need " + xpToLevel + " xp to level up this stat. You only have " + xp + " xp.");
            return false;
        }
        return true;
    }

    @Override
    public StringBuilder run(Player player, List<Player> players, World world) {
        player.upgradeBaseStat(stat);
        return new StringBuilder("Your " + stat + " was increased by 1.");
    }

    @Override
    public String description() {
        return "upgrade one of your stats.\n" +
               "usage: upgrade <stat name>\n";
    }

}