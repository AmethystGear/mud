package server.actions;

import java.util.List;

import server.main.World;
import server.objects.Player;
import server.objects.Player.ReadOnlyPlayer;

public class Attack implements Action {

    @Override
    public boolean matchCommand(String command, int playerID) {
        return command.equals("attack");
    }

    @Override
    public boolean parseCommand(String command, ReadOnlyPlayer player, List<ReadOnlyPlayer> players, World world,
            StringBuilder error) {
        if (player.getMob() == null) {
            error.append("you're not currenty fighting a mob!");
        }
        return player.getMob() != null;
    }

    @Override
    public StringBuilder run(Player player, List<Player> players, World world) {
        StringBuilder out = new StringBuilder("");
        out.append("You attacked " + player.getMob().getBaseStats().get("name") + " and dealt "
                + player.getBaseStats().get("dmg") + " damage.\n");
        player.getMob().changeStat("health", -(Integer) player.getBaseStats().get("dmg"));
        if (player.getMob().isDead()) {
            out.append(player.getMob().getBaseStats().get("name") + ": " + player.getMob().getQuote("player victory")
                    + "\n");
            out.append("You murdered " + player.getMob().getBaseStats().get("name") + "\n");

            out.append("You got " + player.getMob().getBaseStats().get("xp") + " xp.\n");
            player.changeXP((Integer) player.getMob().getBaseStats().get("xp"));

            String[] drops = player.getMob().getDrops();
            for (String drop : drops) {
                out.append("You got " + drop + "\n");
                player.addToInventory(drop, 1);
            }
            world.removeEntity(player.x(), player.y());
            player.setMob(null);
        } else {
            out.append(player.getMob().attack(player, world));
        }
        return out;
    }

    @Override
    public String description() {
        return "attack the mob you are currently interacting with.\n" +
               "usage: attack\n";
    }

}
