import java.io.*;
import java.util.*;

public class World {
    public static final int MAP_SIZE = 3000;
    private int[][] worldMap;
    private int[][] mobMap;
    private Block.BlockSet blocks;
    private int seed;

    public World(String saveFile, int numMobs, Block.BlockSet blocks) throws FileNotFoundException {
        this.blocks = blocks;
        Scanner scan = new Scanner(new File(saveFile));
        seed = scan.nextInt();
        readMap(scan, worldMap);
        readMap(scan, mobMap);
        scan.close();
    }

    public World(int seed, int numMobs, Block.BlockSet blocks) {
        this.blocks = blocks;
        worldMap = new int[MAP_SIZE][];
        mobMap = new int[MAP_SIZE][];
        for(int i = 0; i < MAP_SIZE; i++) {
            worldMap[i] = new int[MAP_SIZE];
            mobMap[i] = new int[MAP_SIZE];
        }

        seed = RandUtils.rand(0, Integer.MAX_VALUE - 1);
        Random rand = new Random(seed);
        float[][] perlinNoise = RandUtils.generatePerlinNoise(MAP_SIZE, MAP_SIZE, rand, 10);
        float waterLevel = 0.5f;
        float sandLevel = 0.53f;
        float grassLevel = 0.75f;
        float tallGrassLevel = 0.85f;
        // create map
        for(int x = 0; x < MAP_SIZE; x++) {
            for(int y = 0; y < MAP_SIZE; y++) {
                int water = blocks.getBlock("water").BLOCK_ID;
                int sand = blocks.getBlock("sand").BLOCK_ID;
                int grass = blocks.getBlock("grass").BLOCK_ID;
                int tallGrass = blocks.getBlock("tall grass").BLOCK_ID;
                int rock = blocks.getBlock("rock").BLOCK_ID;
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
            if(!((String)blocks.getBlock(worldMap[x][y]).STATS.get("name")).contains("water")) {
                spawnVillage(x, y, worldMap, rand, blocks);
            }
        }

        for(int x = 0; x < MAP_SIZE; x++) {
            for(int y = 0; y < MAP_SIZE; y++) {
                Block currentBlock = blocks.getBlock(worldMap[x][y]);
                if(!(Boolean)currentBlock.STATS.get("solid")) {
                    if(currentBlock.STATS.hasVariable("mob-spawn-chance")) {
                        int mobSpawnChance = (Integer)currentBlock.STATS.get("mob-spawn-chance");
                        if(rand.nextInt(100) < mobSpawnChance) {
                            mobMap[x][y] = rand.nextInt(numMobs) + 1;
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
        writer.close();
    }

    private static void saveMap(PrintWriter writer, int[][] map) {
        for(int y = 0; y < MAP_SIZE; y++) {
            for(int x = 0; x < MAP_SIZE; x++) {
                writer.write(map[x][y] + " ");
            }
        }
    }

    private static void readMap(Scanner scan, int[][] map) {
        for(int y = 0; y < MAP_SIZE; y++) {
            for(int x = 0; x < MAP_SIZE; x++) {
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
        return blocks.getBlock(worldMap[x][y]);
    }

    public Mob getMob(int x, int y, String mobFile) {
        if(mobMap[x][y] == 0) {
            return null;
        } else {
            return new Mob(mobMap[x][y], mobFile);
        }
    }

    public boolean hasMob(int x, int y) {
        return mobMap[x][y] != 0;
    }

    private static void spawnVillage(int xOrigin, int yOrigin, int [][] worldMap, Random rand, Block.BlockSet blocks) {
        int floor = blocks.getBlock("village floor").BLOCK_ID;
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

    private static void spawnHut(int xOrigin, int yOrigin, int size, int [][] worldMap, Random rand, Block.BlockSet blocks) {
        int floor = blocks.getBlock("village floor").BLOCK_ID;
        int wall = blocks.getBlock("village wall").BLOCK_ID;
        int surveyor = blocks.getBlock("surveyor").BLOCK_ID;
        int surveyorSpawnChance = (Integer)blocks.getBlock("surveyor").STATS.get("spawn-chance");
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