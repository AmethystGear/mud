package server.actions;

import java.util.List;
import java.util.Scanner;

import server.main.World;
import server.objects.Player;
import server.objects.Player.ReadOnlyPlayer;

public class Give implements Action {
    private int amount;
    private String item;
    private int recipientId;

    @Override
    public boolean matchCommand(String command, int playerID) {
        return command.startsWith("give");
    }

    @Override
    public boolean parseCommand(String command, ReadOnlyPlayer player, List<ReadOnlyPlayer> players, World world,
            StringBuilder error) {
        Scanner scan = new Scanner(command);
        scan.next();
        if(!scan.hasNextInt()) {
            error.append("you need to type the playerId of the person you are giving to!");
            scan.close();
            return false;
        }
        recipientId = scan.nextInt();
        if(!scan.hasNext()) {
            error.append("you need to type the name of the item you are giving!");
            scan.close();
            return false;
        }
        item = scan.next();
        if(!scan.hasNextInt()) {
            error.append("you need to type the amount of the item you are giving!");
            scan.close();
            return false;
        }
        amount = scan.nextInt();
        scan.close();
        if(recipientId < 0 || recipientId >= players.size()) {
            error.append("That player doesn't exist!");
            return false;
        }
        if(!player.getInventory().hasVariable(item)) {
            error.append("You don't have that item!");
            return false;
        }
        if(amount < 0) {
            error.append("Nice try, but you can't give negative donations.");
            return false;
        }
        if((Integer)player.getInventory().get(item) < amount) {
            error.append("You have ");
            error.append((Integer)player.getInventory().get(item) + "");
            error.append(" of that item, not ");
            error.append(amount + ".");
            return false;
        }
        return true;
    }

    @Override
    public StringBuilder run(Player player, List<Player> players, World world) {
        players.get(recipientId).addToInventory(item, amount);
        player.removeFromInventory(item, amount);
        return new StringBuilder("gave " + amount + " " + item + " to player with id " + recipientId);
    }

    @Override
    public String description() {
        return "give items in your inventory to another player.\n" +
               "usage: give <player-id> <item> <number of item>\n";
    }
}
