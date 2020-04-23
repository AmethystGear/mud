package server.actions;

import java.util.List;
import java.util.Scanner;

import server.main.World;
import server.objects.Player;
import server.objects.Player.ReadOnlyPlayer;
import server.objects.Mob;
import server.utils.MathUtils;
import server.utils.RandUtils;
import server.utils.DisplayUtils;



public class Move implements Action {
    
    private boolean xAxis;
    private int numUnits;

    @Override
    public boolean matchCommand(String command) {
        boolean hasWasd = command.charAt(0) == 'w' || command.charAt(0) == 'a' || command.charAt(0) == 's' || command.charAt(0) == 'd';
        if(!hasWasd) {
            return false;
        }
        if(command.length() == 1) {
            return true;
        }
        Scanner scan = new Scanner(command.substring(1, command.length()));
        return scan.hasNextInt();
    }

    @Override
    public boolean parseCommand(String command, ReadOnlyPlayer player, List<ReadOnlyPlayer> players, World world, StringBuilder error) {
        if(player.getMob() != null) {
            error.append("you can't move while fighting a mob!");
            return false;
        }
        xAxis = command.charAt(0) == 'a' || command.charAt(0) == 'd';
        if(command.length() > 1) {
            numUnits = Integer.parseInt(command.substring(1, command.length()));
        } else {
            numUnits = (Integer)player.getStats().get("speed");
        }
        if(numUnits < 0) {
            error.append("you can't move a negative distance!");
            return false;
        }
        if(numUnits > (Integer)player.getStats().get("speed")) {
            error.append("you can't move that far in one turn!");
            return false;
        }
        if(command.charAt(0) == 'a' || command.charAt(0) == 'w') {
            numUnits = -numUnits;
        }
        return true;
    }

    @Override
    public StringBuilder run(Player player, List<Player> players, World world) {
        StringBuilder s = new StringBuilder("");
        int newPosn = move(player.x(), player.y(), xAxis, numUnits, world);
        if(xAxis) {
            player.moveTo(newPosn, player.y());
        } else {
            player.moveTo(player.x(), newPosn);
        }
        int viewDist = (Integer)player.getStats().get("view");
        s.append(DisplayUtils.display(viewDist, player.x(), player.y(), players, world, true));
        s.append("\n");

        if(world.hasMob(player.x(), player.y())) {
            Mob mob = world.getMob(player.x(), player.y());
            s.append("You encountered: ");
            s.append(mob.getBaseStats().get("name") + "\n");
            player.setMob(mob);
            s.append(mob.getImg() + "\n");
            s.append(mob.getQuote("entrance") + "\n");
            int playerSpeed = (Integer)player.getStats().get("speed");
            int mobSpeed = (Integer)mob.getStats().get("speed");
            if(playerSpeed < mobSpeed) {
                s.append(mob.attack(player, world));
                s.append("\n");
            }
        }
        return s;
    }

    // calculates the actual move position given the origin, direction to move in, distance to attempt to travel,
    // the mob map, and the world map.
    private static int move(int xOrigin, int yOrigin, boolean xAxis, int numUnits, World world) {
        if(numUnits == 0 && xAxis) {
            return xOrigin;
        } else if (numUnits == 0 && !xAxis) {
            return yOrigin;
        }

        if(xAxis) {
            int bounded = MathUtils.bound(xOrigin + numUnits, 0, World.MAP_SIZE - 1);
            for(int x = xOrigin + MathUtils.sign(numUnits); x != bounded; x += MathUtils.sign(numUnits)) {
                if(world.getBlock(x, yOrigin).getStats().hasProperty("solid")) {
                    return x - MathUtils.sign(numUnits);
                } else if(world.hasMob(x, yOrigin)) {
                    Mob m = world.getMob(x, yOrigin);
                    int agg;
                    if(!m.getBaseStats().hasVariable("aggression")) {
                        agg = 0;
                    } else {
                        agg = (Integer)m.getBaseStats().get("aggression");
                    }

                    if(RandUtils.rand(0, 99) < agg) {
                        System.out.println("you were blocked by a mob!");
                        return x;
                    }
                }
            }
            boolean solid = (Boolean)world.getBlock(bounded, yOrigin).getStats().hasProperty("solid");
            return solid ? bounded - MathUtils.sign(numUnits) : bounded;
        } else {
            int bounded = MathUtils.bound(yOrigin + numUnits, 0, World.MAP_SIZE - 1);
            for(int y = yOrigin + MathUtils.sign(numUnits); y != bounded; y += MathUtils.sign(numUnits)) {
                if(world.getBlock(xOrigin, y).getStats().hasProperty("solid")) {
                    return y - MathUtils.sign(numUnits);
                } else if(world.hasMob(xOrigin, y)) {
                    Mob m = world.getMob(xOrigin, y);

                    int agg;
                    if(!m.getBaseStats().hasVariable("aggression")) {
                        agg = 0;
                    } else {
                        agg = (Integer)m.getBaseStats().get("aggression");
                    }
                    
                    if(RandUtils.rand(0, 99) < agg) {
                        System.out.println("you were blocked by a mob!");
                        return y;
                    }
                }
            }
            boolean solid = (Boolean)world.getBlock(xOrigin, bounded).getStats().hasProperty("solid");
            return solid ? bounded - MathUtils.sign(numUnits) : bounded;
        }
    }
}