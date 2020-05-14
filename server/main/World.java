package server.main;

import java.io.*;
import java.util.*;

import server.objects.Mob;
import server.objects.Item;
import server.objects.Block;
import server.objects.Value;
import server.utils.MultiDimensionalFloatArray;
import server.utils.MultiDimensionalIntArray;
import server.utils.RandUtils;

public class World {
    public static final int MAP_SIZE = 3000;
    private MultiDimensionalIntArray worldMap;
    private MultiDimensionalIntArray mobMap;

    public final Value.ValueSet<Mob.ReadOnlyMob> mobs;
    private final String mobsEndString = "/mobs/";
    public final Value.ValueSet<Item> items;
    private final String itemsEndString = "/items/";
    public final Value.ValueSet<Block> blocks;
    private final String blocksEndString = "/blocks/";

    private final String pathToMobsConfig = "config/mobs.txt";
    private final String pathToItemsConfig = "config/items.txt";
    private final String pathToBlocksConfig = "config/blocks.txt";

    private int seed;

    public World(String saveFile) throws FileNotFoundException {
        worldMap = new MultiDimensionalIntArray(MAP_SIZE, MAP_SIZE);
        mobMap = new MultiDimensionalIntArray(MAP_SIZE, MAP_SIZE);

        Scanner scan = new Scanner(new File(saveFile));
        seed = scan.nextInt();
        System.out.println(seed);
        readMap(scan, worldMap);
        readMap(scan, mobMap);
        scan.nextLine();
        mobs = Value.getValuesFromScanner(scan, mobsEndString, new Mob.ReadOnlyMob(new Mob()));
        items = Value.getValuesFromScanner(scan, itemsEndString, new Item());
        blocks = Value.getValuesFromScanner(scan, blocksEndString, new Block());
        scan.close();
    }

    public World(int seed) throws FileNotFoundException {
        this.seed = seed;
        worldMap = new MultiDimensionalIntArray(MAP_SIZE, MAP_SIZE);
        mobMap = new MultiDimensionalIntArray(MAP_SIZE, MAP_SIZE);

        mobs = Value.getValuesFromScanner(new Scanner(new File(pathToMobsConfig)), new Mob.ReadOnlyMob(new Mob()));
        items = Value.getValuesFromScanner(new Scanner(new File(pathToItemsConfig)), new Item());
        blocks = Value.getValuesFromScanner(new Scanner(new File(pathToBlocksConfig)), new Block());

        Random rand = new Random(seed);
        MultiDimensionalFloatArray perlinNoise = RandUtils.generatePerlinNoise(MAP_SIZE, MAP_SIZE, rand, 10);

        float waterLevel = 0.5f;
        float sandLevel = 0.53f;
        float grassLevel = 0.75f;
        float tallGrassLevel = 0.85f;
        // create map
        for (int i = 0; i < MAP_SIZE * MAP_SIZE; i++) {
            int water = blocks.get("water").getID();
            int sand = blocks.get("sand").getID();
            int grass = blocks.get("grass").getID();
            int tallGrass = blocks.get("forest").getID();
            int rock = blocks.get("rock").getID();
            int block = 0;
            float height = perlinNoise.get(i);
            if (height < waterLevel) {
                block = water;
            } else if (height >= waterLevel && height < sandLevel) {
                block = sand;
            } else if (height >= sandLevel && height < grassLevel) {
                block = grass;
            } else if (height >= grassLevel && height < tallGrassLevel) {
                block = tallGrass;
            } else {
                block = rock;
            }
            worldMap.set(i, block);
        }

        int numVillages = rand.nextInt(50) + 50;
        for (int i = 0; i < numVillages; i++) {
            int x = rand.nextInt(2000) + 500;
            int y = rand.nextInt(2000) + 500;
            if (!((String) blocks.get(worldMap.get(x, y)).getStats().get("name")).contains("water")) {
                spawnVillage(x, y, rand, blocks);
            }
        }

        for (int i = 0; i < MAP_SIZE * MAP_SIZE; i++) {
            Block currentBlock = blocks.get(worldMap.get(i));
            mobMap.set(-1, i);
            if (!currentBlock.getStats().hasProperty("solid")) {
                if (currentBlock.getStats().hasVariable("mob spawn chance")) {
                    double mobSpawnChance = (Double) currentBlock.getStats().get("mob spawn chance");
                    if (rand.nextDouble() < mobSpawnChance) {
                        mobMap.set(rand.nextInt(mobs.size()), i);
                    }
                }
            }
        }
    }

    public void saveTo(String file) throws FileNotFoundException {
        PrintWriter writer = new PrintWriter(new FileOutputStream(new File(file)));
        writer.write(seed + " ");
        saveMap(writer, worldMap);
        saveMap(writer, mobMap);
        writer.write("\n");
        for (Mob.ReadOnlyMob m : mobs.values()) {
            m.getBaseStats().saveTo(writer);
            writer.write("\n");
        }
        writer.write(mobsEndString);
        writer.write("\n");
        for (Item i : items.values()) {
            i.getStats().saveTo(writer);
            writer.write("\n");
        }
        writer.write(itemsEndString);
        writer.write("\n");
        for (Block b : blocks.values()) {
            b.getStats().saveTo(writer);
            writer.write("\n");
        }
        writer.write(blocksEndString);
        writer.write("\n");
        writer.close();
    }

    private static void saveMap(PrintWriter writer, MultiDimensionalIntArray map) {
        for (int x = 0; x < MAP_SIZE * MAP_SIZE; x++) {
            writer.write(map.get(x) + " ");
        }
    }

    private static void readMap(Scanner scan, MultiDimensionalIntArray map) {
        for (int x = 0; x < MAP_SIZE * MAP_SIZE; x++) {
            map.set(scan.nextInt(), x);
        }
    }

    public Random getXYRand(int x, int y) {
        long hash = x * (int) Math.pow(10, (x + "").length()) + y;
        hash += seed;
        return new Random(hash);
    }

    public Block getBlock(int x, int y) {
        return blocks.get(worldMap.get(x, y));
    }

    public Mob getMob(int x, int y) {
        if (mobMap.get(x, y) == -1) {
            return null;
        } else {
            // make a new mob with exactly the same properties as the read only mob, and return it.
            Mob.ReadOnlyMob baseMob = mobs.get(mobMap.get(x, y));
            return baseMob.instantiateClone();
        }
    }

    public boolean hasMob(int x, int y) {
        return mobMap.get(x, y) != -1;
    }

    public void removeMob(int x, int y) {
        mobMap.set(-1, x, y);
    }

    private void spawnVillage(int xOrigin, int yOrigin, Random rand, Value.ValueSet<Block> blocks) {
        int floor = blocks.get("village floor").getID();
        int villageLength = rand.nextInt(100) + 20;
        int pathSize = rand.nextInt(2) + 3;
        for (int x = xOrigin; x < xOrigin + villageLength; x++) {
            for (int y = yOrigin; y < yOrigin + pathSize; y++) {
                worldMap.set(floor, x, y);
            }
        }
        boolean generateUp = false;
        for (int x = xOrigin + rand.nextInt(3) + 2; x < xOrigin + villageLength; x += (rand.nextInt(5) + 5)) {
            int pathlen = rand.nextInt(7) + 3;
            int hutSize = rand.nextInt(2) + 2;
            if (generateUp) {
                for (int y = yOrigin; y > yOrigin - pathlen; y--) {
                    worldMap.set(floor, x, y);
                }
                spawnHut(x - hutSize, yOrigin - pathlen - hutSize * 2 + 1, hutSize, rand, blocks);
            } else {
                for (int y = yOrigin + pathSize; y < yOrigin + pathlen + pathSize; y++) {
                    worldMap.set(floor, x, y);
                }
                spawnHut(x - hutSize, yOrigin + pathlen + pathSize, hutSize, rand, blocks);
            }
            generateUp = !generateUp;
        }
    }

    private void spawnHut(int xOrigin, int yOrigin, int size, Random rand, Value.ValueSet<Block> blocks) {
        int floor = blocks.get("village floor").getID();
        int wall = blocks.get("village wall").getID();
        size = size * 2 + 1;
        for (int x = xOrigin; x < xOrigin + size; x++) {
            for (int y = yOrigin; y < yOrigin + size; y++) {
                worldMap.set(floor, x, y);
            }
        }
        for (int x = xOrigin; x < xOrigin + size; x++) {
            if (x - xOrigin != size / 2) {
                worldMap.set(wall, x, yOrigin);
                worldMap.set(wall, x, yOrigin + size - 1);
            }
        }
        for (int y = yOrigin; y < yOrigin + size; y++) {
            worldMap.set(wall, xOrigin, y);
            worldMap.set(wall, xOrigin + size - 1, y);
        }
    }
}
