package server.utils;

import java.util.List;
import java.util.ArrayList;

import server.main.World;
import server.objects.Block;
import server.objects.Player;

// contains static methods used to display the world
public class DisplayUtils {

    /** 
     * creates a string rep of the world map and returns the StringBuilder containing it.
     * 
     * @throws IllegalArgumentException if players or world is null.
     * @param chunkSize chunkSize^2 is the amount of blocks each pixel on the map represents
     * @param players all the players we want to appear on the map.
     * @param world the world
     * @return StringBuilder that contains the entire map
     */
    public static StringBuilder map(int chunkSize, List<Player> players, World world) {
        if(players == null || world == null) {
            throw new IllegalArgumentException();
        }
        StringBuilder s = new StringBuilder();
        for (int y = 0; y < World.MAP_SIZE; y += chunkSize) {
            s.append("|");
            int prevAscii = -1;
            for (int x = 0; x < World.MAP_SIZE; x += chunkSize) {
                boolean hasPlayer = false;
                for (Player player : players) {
                    if (player.x() >= x && player.x() < x + chunkSize && player.y() > y
                            && player.y() <= y + chunkSize) {
                        s.append("\033[0m" + player.toString());
                        prevAscii = -1;
                        hasPlayer = true;
                        break;
                    }
                }
                if (!hasPlayer) {
                    int majorityBlock = getMajorityBlockInChunk(x, y, chunkSize, world);
                    int asciiColor = (Integer) world.blocks.get(majorityBlock).getStats().get("display");
                    if (asciiColor != -1 && asciiColor != prevAscii) {
                        s.append("\033[48;5;" + asciiColor + "m");
                    }
                    s.append("  ");
                    prevAscii = asciiColor;
                }
            }
            s.append("\033[0m|\n");
        }
        return s;
    }

    /**
     * Return most common block in this chunk of the map.
     * NOTE: each block has it's own map weight, which determines how much it influences the return value of this function.
     * a low map weight means that even high amounts of the block per chunk wouldn't be visibile, a high map weight means even
     * low amounts of blocks per chunk would become visible on the map.
     * 
     * @throws IllegalArgumentException if world is null
     * @param xOrigin x position of the start of the chunk
     * @param yOrigin y position of the start of the chunk
     * @param chunkSize size of the chunk
     * @param world the world
     * @return int that represents the most common block id in the chunk (influenced by map-weight).
     */
    private static int getMajorityBlockInChunk(int xOrigin, int yOrigin, int chunkSize, World world) {
        if(world == null) {
            throw new IllegalArgumentException();
        }
        ArrayList<Integer> blockList = new ArrayList<Integer>();
        for (int x = xOrigin; x < xOrigin + chunkSize; x++) {
            for (int y = yOrigin; y < yOrigin + chunkSize; y++) {
                Block b = world.getBlock(x, y);
                while (blockList.size() <= b.getID()) {
                    blockList.add(0);
                }
                int mapWeight;
                if (b.getStats().hasVariable("map weight")) {
                    mapWeight = (Integer) b.getStats().get("map weight");
                } else {
                    mapWeight = 1;
                }
                blockList.set(b.getID(), blockList.get(b.getID()) + mapWeight);
            }
        }
        int maxIndex = 0;
        for (int i = 1; i < blockList.size(); i++) {
            if (blockList.get(i) > blockList.get(maxIndex)) {
                maxIndex = i;
            }
        }
        return maxIndex;
    }

    /**
     * creates a square, top-down view of a portion of the map and returns the StringBuilder containing it.
     * 
     * @throws IllegalArgumentException if players or world is null
     * @param dist the distance from the center of the view that we are going to display
     * @param xView the x center position of the view
     * @param yView the y center position of the view
     * @param players all the players we could potentially display (if they are within the 'dist' range)
     * @param world the world
     * @param showMobs should we display mobs or not?
     * @return StringBuilder that contains a square, top-down view of the world centered at (xView, yView) and extending by dist.
     */
    public static StringBuilder display(int dist, int xView, int yView, List<Player> players, World world,
            boolean showMobs) {
        
        if(players == null || world == null) {
            throw new IllegalArgumentException();
        }

        StringBuilder s = new StringBuilder();
        for (int y = MathUtils.max(0, yView - dist); y < MathUtils.min(World.MAP_SIZE, yView + dist + 1); y++) {
            s.append("|");
            int prevAscii = -1;
            for (int x = MathUtils.max(0, xView - dist); x < MathUtils.min(World.MAP_SIZE, xView + dist + 1); x++) {
                int leastDist = Integer.MAX_VALUE;
                boolean hasPlayer = false;
                for (Player player : players) {
                    int manhattanDist = MathUtils.manhattan(x, y, player.x(), player.y());
                    if (manhattanDist < leastDist) {
                        leastDist = manhattanDist;
                    }
                    if (player.x() == x && player.y() == y) {
                        s.append("\033[0m" + player.toString());
                        prevAscii = -1;
                        hasPlayer = true;
                        break;
                    }
                }
                if (!hasPlayer) {
                    Block b = world.getBlock(x, y);
                    if(b.display() != prevAscii) {
                        s.append("\033[48;5;" + b.display() + "m");
                    }
                    if (world.hasMob(x, y) && showMobs) {
                        s.append("??");
                    } else {
                        s.append("  ");
                    }
                    prevAscii = b.display();
                }
            }
            s.append("\033[0m|\n");
        }
        return s;
    }
}