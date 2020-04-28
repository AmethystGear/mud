package server.actions;

import java.util.List;
import java.util.Random;
import java.util.Scanner;

import server.main.World;
import server.objects.Player;
import server.objects.Player.ReadOnlyPlayer;
import server.objects.Item;

public class Trade implements Action {

    int numToTrade;
    int tradeNumber;

    @Override
    public boolean matchCommand(String command) {
        return command.startsWith("trade");
    }

    @Override
    public boolean parseCommand(String command, ReadOnlyPlayer player, List<ReadOnlyPlayer> players, World world,
            StringBuilder error) {
        if(player.getMob() == null) {
            error.append("you aren't currently interacting with a mob!");
            return false;
        }
        if(!player.getMob().getBaseStats().hasProperty("trades")) {
            error.append("you can't trade with " + (String)player.getMob().getBaseStats().get("name"));
            return false;
        }
        if(command.equals("trade")) {
            return true;
        }
        Scanner scan = new Scanner(command);
        if(!scan.hasNextInt()) {
            error.append("You need to type which number trade you are making!");
            scan.close();
            return false;
        }
        tradeNumber = scan.nextInt();
        if(!scan.hasNextInt()) {
            error.append("You need to type the amount of items you are going to trade!");
            scan.close();
            return false;
        }
        numToTrade = scan.nextInt();
        scan.close();
        int numItems = (Integer)player.getMob().getBaseStats().get("num trade items");
        if(tradeNumber < 1 || tradeNumber > numItems) {
            error.append("the trade number must be between 1 and " + numItems + "!");
            return false;
        }

        Random random = world.getXYRand(player.x(), player.y());
        Item [] itemsToTrade = new Item[numItems];
        for(int i = 0; i < numItems; i++) {
            itemsToTrade[i] = world.items.get(random.nextInt(world.items.size()));
        }
        String itemName = (String)itemsToTrade[tradeNumber].getStats().get("name");
        int amountOfItemToTrade = player.getInventory().hasVariable(itemName) ? ((Integer)player.getInventory().get("name")) : 0;
        if(numToTrade > amountOfItemToTrade) {
            error.append("You only have " + amountOfItemToTrade + " of that item.");
            return false;
        }                
        return true;
    }

    @Override
    public StringBuilder run(Player player, List<Player> players, World world) {
        Random random = world.getXYRand(player.x(), player.y());
        int numItems = (Integer)player.getMob().getBaseStats().get("num trade items");
        Item [] itemsToTrade = new Item[numItems];
        for(int i = 0; i < numItems; i++) {
            itemsToTrade[i] = world.items.get(random.nextInt(world.items.size()));
        }

        int [] baseValueBonus = new int[numItems];
        if(player.getMob().getBaseStats().hasVariable("max base value bonus")) {
            int maxBonus = (Integer)player.getMob().getBaseStats().get("max base value bonus");
            for(int i = 0; i < numItems; i++) {
                baseValueBonus[i] = random.nextInt(maxBonus);
            }
        }

        StringBuilder out = new StringBuilder();
        if(tradeNumber == 0) {
            out.append("I can give: \n");
            for(int i = 0; i < numItems; i++) {
                String itemName = (String)itemsToTrade[i].getStats().get("name");
                int baseValue = (Integer)itemsToTrade[i].getStats().get("base value");
                out.append((i + 1) + ". " + (baseValue + baseValueBonus[i]) + " xp for " +  itemName + "\n");
            }
            out.append("type 'trade <trade-number> <trade-amount>' to trade.\n");
        } else {
            player.removeFromInventory((String)itemsToTrade[tradeNumber].getStats().get("name"), numToTrade);
            int baseValue = (Integer)itemsToTrade[tradeNumber].getStats().get("base value");
            player.changeStat("xp", numToTrade * (baseValue + baseValueBonus[tradeNumber]));
            out.append("you got " + (numToTrade * (baseValue + baseValueBonus[tradeNumber])) + " xp.\n");
        }
        return out;
    }

    @Override
    public String description() {
        return "trade with the mob you are currently interacting with.\n" +
               "usage: trade | trade <trade number> <amount to trade>\n" +
               "example: trade (this would list all the trades the mob can make)\n" +
               "example: trade 1 5 (this would do 5 of the first trade)\n";
    }

}