package server.main;

import java.io.*;
import java.util.*;

import server.objects.Mob;
import server.objects.Item;
import server.objects.Block;
import server.objects.Value;

import server.utils.RandUtils;

public class World {
    public static final int MAP_SIZE = 3000;
    private int[][] worldMap;
    private int[][] mobMap;

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
        worldMap = new int[MAP_SIZE][];
        mobMap = new int[MAP_SIZE][];
        for(int i = 0; i < MAP_SIZE; i++) {
            worldMap[i] = new int[MAP_SIZE];
            mobMap[i] = new int[MAP_SIZE];
        }
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
        worldMap = new int[MAP_SIZE][];
        mobMap = new int[MAP_SIZE][];
        for(int i = 0; i < MAP_SIZE; i++) {
            worldMap[i] = new int[MAP_SIZE];
            mobMap[i] = new int[MAP_SIZE];
        }

        mobs = Value.getValuesFromScanner(new Scanner(new File(pathToMobsConfig)), new Mob.ReadOnlyMob(new Mob()));
        items = Value.getValuesFromScanner(new Scanner(new File(pathToItemsConfig)), new Item());
        blocks = Value.getValuesFromScanner(new Scanner(new File(pathToBlocksConfig)), new Block());



        Random rand = new Random(seed);
        float[][] baseHeightMap = RandUtils.generatePerlinNoise(MAP_SIZE, MAP_SIZE, rand, 10);
        float[][] secondLayerHeightMap = RandUtils.generatePerlinNoise(MAP_SIZE, MAP_SIZE, rand, rand.nextInt(3) + 3);
        float[][] perlinNoise = new float[MAP_SIZE][];

        float secondaryWeight = 0.2f;
        for(int i = 0; i < MAP_SIZE; i++) {
            perlinNoise[i] = new float[MAP_SIZE];
            for(int j = 0; j < MAP_SIZE; j++) {
                perlinNoise[i][j] = (baseHeightMap[i][j] + secondLayerHeightMap[i][j] * secondaryWeight);
            }
        }

        float waterLevel = 0.5f;
        float sandLevel = 0.53f;
        float grassLevel = 0.75f;
        float tallGrassLevel = 0.85f;
        // create map
        for(int x = 0; x < MAP_SIZE; x++) {
            for(int y = 0; y < MAP_SIZE; y++) {
                int water = blocks.get("water").getID();
                int sand = blocks.get("sand").getID();
                int grass = blocks.get("grass").getID();
                int tallGrass = blocks.get("forest").getID();
                int rock = blocks.get("rock").getID();
                int block = 0;
                if(perlinNoise[x][y] < waterLevel) {
                    block = water;
                } else if (perlinNoise[x][y] >= waterLevel && perlinNoise[x][y] < sandLevel) {
                    block = sand;
                } else if  (perlinNoise[x][y] >= sandLevel && perlinNoise[x][y] < grassLevel) {
                    block = grass;
                } else if  (perlinNoise[x][y] >= grassLevel && perlinNoise[x][y] < tallGrassLevel){
                    block = tallGrass;
                } else {
                    block = rock;
                }
                worldMap[x][y] = block;
            }
        }

        int numVillages = rand.nextInt(50) + 50;
        for(int i = 0; i < numVillages; i++) {
            int x = rand.nextInt(2000) + 500;
            int y = rand.nextInt(2000) + 500;
            if(!((String)blocks.get(worldMap[x][y]).getStats().get("name")).contains("water")) {
                spawnVillage(x, y, worldMap, rand, blocks);
            }
        }

        for(int x = 0; x < MAP_SIZE; x++) {
            for(int y = 0; y < MAP_SIZE; y++) {
                mobMap[x][y] = -1;
                Block currentBlock = blocks.get(worldMap[x][y]);
                if(!currentBlock.getStats().hasProperty("solid")) {
                    if(currentBlock.getStats().hasVariable("mob spawn chance")) {
                        double mobSpawnChance = (Double)currentBlock.getStats().get("mob spawn chance");
                        if(rand.nextDouble() < mobSpawnChance) {
                            mobMap[x][y] = rand.nextInt(mobs.size());
                        }
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
        for(Mob.ReadOnlyMob m : mobs.values()) {
            m.getBaseStats().saveTo(writer);
            writer.write("\n");
        }
        writer.write(mobsEndString);
        writer.write("\n");
        for(Item i : items.values()) {
            i.getStats().saveTo(writer);
            writer.write("\n");
        }
        writer.write(itemsEndString);
        writer.write("\n");
        for(Block b : blocks.values()) {
            b.getStats().saveTo(writer);
            writer.write("\n");
        }
        writer.write(blocksEndString);
        writer.write("\n");
        writer.close();
    }

    private static void saveMap(PrintWriter writer, int[][] map) {
        for(int x = 0; x < MAP_SIZE; x++) {
            for(int y = 0; y < MAP_SIZE; y++) {
                writer.write(map[x][y] + " ");
            }
        }
    }

    private static void readMap(Scanner scan, int[][] map) {
        for(int x = 0; x < MAP_SIZE; x++) {
            for(int y = 0; y < MAP_SIZE; y++) {
                map[x][y] = scan.nextInt();
            }
        }
    }

    public Random getXYRand(int x, int y) {
        long hash = x * (int)Math.pow(10, (x + "").length()) + y;
        hash += seed;
        return new Random(hash);
    }

    public Block getBlock(int x, int y) {
        return blocks.get(worldMap[x][y]);
    }

    public Mob getMob(int x, int y) {
        if(mobMap[x][y] == -1) {
            return null;
        } else {
            // make a new mob with exactly the same properties as the read only mob, and return it.
            Mob.ReadOnlyMob baseMob = mobs.get(mobMap[x][y]);
            return baseMob.clone();
        }
    }

    public boolean hasMob(int x, int y) {
        return mobMap[x][y] != -1;
    }

    public void removeMob(int x, int y) {
        mobMap[x][y] = -1;
    }

    private static void spawnVillage(int xOrigin, int yOrigin, int [][] worldMap, Random rand, Value.ValueSet<Block> blocks) {
        int floor = blocks.get("village floor").getID();
        int villageLength = rand.nextInt(100) + 20;
        int pathSize = rand.nextInt(2) + 3;
        for(int x = xOrigin; x < xOrigin + villageLength; x++) {
            for(int y = yOrigin; y < yOrigin + pathSize; y++) {
                worldMap[x][y] = floor;
            }
        }
        boolean generateUp = false;
        for(int x = xOrigin + rand.nextInt(3) + 2; x < xOrigin + villageLength; x+= (rand.nextInt(5) + 5)) {
            int pathlen = rand.nextInt(7) + 3;
            int hutSize = rand.nextInt(2) + 2;
            if(generateUp) {
                for(int y = yOrigin; y > yOrigin - pathlen; y--) {
                    worldMap[x][y] = floor;
                }
                spawnHut(x - hutSize, yOrigin - pathlen - hutSize * 2 + 1, hutSize, worldMap, rand, blocks);
            } else {
                for(int y = yOrigin + pathSize; y < yOrigin + pathlen + pathSize; y++) {
                    worldMap[x][y] = floor;
                }
                spawnHut(x - hutSize, yOrigin + pathlen + pathSize, hutSize, worldMap, rand, blocks);
            }
            generateUp = !generateUp;
        }
    }

    private static void spawnHut(int xOrigin, int yOrigin, int size, int [][] worldMap, Random rand, Value.ValueSet<Block> blocks) {
        int floor = blocks.get("village floor").getID();
        int wall = blocks.get("village wall").getID();
        int surveyor = blocks.get("surveyor").getID();
        int surveyorSpawnChance = 0;
        size = size * 2 + 1;
        for(int x = xOrigin; x < xOrigin + size; x++) {
            for(int y = yOrigin; y < yOrigin + size; y++) {
                worldMap[x][y] = floor;
                if(rand.nextInt(100) < surveyorSpawnChance) {
                    worldMap[x][y] = surveyor;
                }
            }
        }
        for(int x = xOrigin; x < xOrigin + size; x++) {
            if(x - xOrigin != size/2) {
                worldMap[x][yOrigin] = wall;
                worldMap[x][yOrigin + size - 1] = wall;
            }
        }
        for(int y = yOrigin; y < yOrigin + size; y++) {
            worldMap[xOrigin][y] = wall;
            worldMap[xOrigin + size - 1][y] = wall;
        }
    }
}   