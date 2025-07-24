import { faker } from '@faker-js/faker';
import { z } from 'zod';

// Zod schemas for validation
const ThingSpeakDataSchema = z.object({
    field1: z.union([z.string(), z.number()]).optional(), // Proof of views
    field2: z.union([z.string(), z.number()]).optional(), // Proof of taps
    field3: z.union([z.string(), z.number()]).optional(),
    field4: z.union([z.string(), z.number()]).optional(),
    field5: z.union([z.string(), z.number()]).optional(),
    field6: z.union([z.string(), z.number()]).optional(),
    field7: z.union([z.string(), z.number()]).optional(),
    field8: z.union([z.string(), z.number()]).optional(),
    lat: z.number().optional(),
    long: z.number().optional(),
    elevation: z.number().optional(),
    status: z.string().optional(),
});

const ThingSpeakResponseSchema = z.object({
    channel_id: z.number().optional(),
    entry_id: z.number().optional(),
    created_at: z.string().optional(),
    field1: z.string().optional(),
    field2: z.string().optional(),
});

const ThingSpeakConfigSchema = z.object({
    apiKey: z.string().min(1, "API key is required"),
    channelId: z.number().default(2890626), // Your AdNet Module View test channel
    baseUrl: z.string().url().optional(),
});

// Schema for reading channel data
const ThingSpeakChannelFeedSchema = z.object({
    created_at: z.string(),
    entry_id: z.number(),
    field1: z.string().nullable().optional(),
    field2: z.string().nullable().optional(),
});

const ThingSpeakChannelSchema = z.object({
    channel: z.object({
        id: z.number(),
        name: z.string(),
        description: z.string(),
        latitude: z.string(),
        longitude: z.string(),
        field1: z.string(),
        field2: z.string(),
        created_at: z.string(),
        updated_at: z.string(),
        last_entry_id: z.number(),
    }),
    feeds: z.array(ThingSpeakChannelFeedSchema),
});

// Infer types from schemas
type ThingSpeakData = z.infer<typeof ThingSpeakDataSchema>;
type ThingSpeakResponse = z.infer<typeof ThingSpeakResponseSchema>;
type ThingSpeakConfig = z.infer<typeof ThingSpeakConfigSchema>;
type ThingSpeakChannelFeed = z.infer<typeof ThingSpeakChannelFeedSchema>;
type ThingSpeakChannel = z.infer<typeof ThingSpeakChannelSchema>;

class ThingSpeakSender {
    private config: ThingSpeakConfig;
    private baseUrl: string;

    constructor(config: ThingSpeakConfig) {
        // Validate configuration using Zod
        this.config = ThingSpeakConfigSchema.parse(config);
        this.baseUrl = config.baseUrl || 'https://api.thingspeak.com';
    }

    /**
     * Validate data before sending
     */
    private validateData(data: ThingSpeakData): ThingSpeakData {
        return ThingSpeakDataSchema.parse(data);
    }

    /**
     * Send data to ThingSpeak using URL parameters (GET/POST)
     */
    async sendDataViaParams(data: ThingSpeakData): Promise<ThingSpeakResponse> {
        // Validate input data
        const validatedData = this.validateData(data);

        const params = new URLSearchParams();
        params.append('api_key', this.config.apiKey);

        // Add fields to parameters
        Object.entries(validatedData).forEach(([key, value]) => {
            if (value !== undefined && value !== null) {
                params.append(key, value.toString());
            }
        });

        const url = `${this.baseUrl}/update?${params.toString()}`;

        try {
            const response = await fetch(url, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/x-www-form-urlencoded',
                },
            });

            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            const result = await response.text();

            // ThingSpeak returns entry_id as plain text on success, 0 on failure
            const responseData = {
                entry_id: parseInt(result) || 0,
                created_at: new Date().toISOString(),
            };

            // Validate and return response
            return ThingSpeakResponseSchema.parse(responseData);
        } catch (error) {
            if (error instanceof z.ZodError) {
                console.error('Response validation error:', error.errors);
                throw new Error(`Invalid response format: ${error.errors.map(e => e.message).join(', ')}`);
            }
            console.error('Error sending data via params:', error);
            throw error;
        }
    }

    /**
     * Send data to ThingSpeak using JSON body
     */
    async sendDataViaJSON(data: ThingSpeakData): Promise<ThingSpeakResponse> {
        // Validate input data
        const validatedData = this.validateData(data);

        const payload = {
            api_key: this.config.apiKey,
            ...validatedData,
        };

        try {
            const response = await fetch(`${this.baseUrl}/update.json`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(payload),
            });

            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            const result = await response.json();

            // Validate and return response
            return ThingSpeakResponseSchema.parse(result);
        } catch (error) {
            if (error instanceof z.ZodError) {
                console.error('Response validation error:', error.errors);
                throw new Error(`Invalid response format: ${error.errors.map(e => e.message).join(', ')}`);
            }
            console.error('Error sending data via JSON:', error);
            throw error;
        }
    }

    /**
     * Send bulk data to ThingSpeak (multiple entries)
     */
    async sendBulkData(dataArray: ThingSpeakData[], delayMs: number = 15000): Promise<void> {
        console.log(`Sending ${dataArray.length} entries to channel ${this.config.channelId} with ${delayMs}ms delay between requests...`);

        for (let i = 0; i < dataArray.length; i++) {
            try {
                const result = await this.sendDataViaParams(dataArray[i]);
                console.log(`Entry ${i + 1}/${dataArray.length} sent successfully:`, result);

                // ThingSpeak has a rate limit of 1 request per 15 seconds for free accounts
                if (i < dataArray.length - 1) {
                    console.log(`Waiting ${delayMs}ms before next request...`);
                    await this.delay(delayMs);
                }
            } catch (error) {
                console.error(`Failed to send entry ${i + 1}:`, error);
            }
        }
    }

    /**
     * Read data from your ThingSpeak channel
     */
    async readChannelData(results: number = 100): Promise<ThingSpeakChannel> {
        const url = `${this.baseUrl}/channels/${this.config.channelId}/feeds.json?results=${results}`;

        try {
            const response = await fetch(url);

            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            const result = await response.json();

            // Validate and return channel data
            return ThingSpeakChannelSchema.parse(result);
        } catch (error) {
            if (error instanceof z.ZodError) {
                console.error('Channel data validation error:', error.errors);
                throw new Error(`Invalid channel data format: ${error.errors.map(e => e.message).join(', ')}`);
            }
            console.error('Error reading channel data:', error);
            throw error;
        }
    }

    /**
     * Read specific field data from your channel
     */
    async readFieldData(fieldNumber: 1 | 2, results: number = 100): Promise<{ feeds: ThingSpeakChannelFeed[] }> {
        const url = `${this.baseUrl}/channels/${this.config.channelId}/fields/${fieldNumber}.json?results=${results}`;

        try {
            const response = await fetch(url);

            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            const result = await response.json();

            // Validate feeds array
            const feedsSchema = z.object({
                feeds: z.array(ThingSpeakChannelFeedSchema)
            });

            return feedsSchema.parse(result);
        } catch (error) {
            if (error instanceof z.ZodError) {
                console.error('Field data validation error:', error.errors);
                throw new Error(`Invalid field data format: ${error.errors.map(e => e.message).join(', ')}`);
            }
            console.error('Error reading field data:', error);
            throw error;
        }
    }

    /**
     * Get channel statistics
     */
    async getChannelStats(): Promise<{
        totalEntries: number;
        avgViews: number;
        avgTaps: number;
        maxViews: number;
        maxTaps: number;
        lastUpdate: string;
    }> {
        try {
            const channelData = await this.readChannelData();

            const views = channelData.feeds
                .map(feed => parseInt(feed.field1 || '0'))
                .filter(val => !isNaN(val));

            const taps = channelData.feeds
                .map(feed => parseInt(feed.field2 || '0'))
                .filter(val => !isNaN(val));

            return {
                totalEntries: channelData.feeds.length,
                avgViews: views.length > 0 ? Math.round(views.reduce((a, b) => a + b, 0) / views.length) : 0,
                avgTaps: taps.length > 0 ? Math.round(taps.reduce((a, b) => a + b, 0) / taps.length) : 0,
                maxViews: views.length > 0 ? Math.max(...views) : 0,
                maxTaps: taps.length > 0 ? Math.max(...taps) : 0,
                lastUpdate: channelData.channel.updated_at,
            };
        } catch (error) {
            console.error('Error calculating channel stats:', error);
            throw error;
        }
    }

    private delay(ms: number): Promise<void> {
        return new Promise(resolve => setTimeout(resolve, ms));
    }
}

// Fake data generator using Faker.js
class FakeDataGenerator {
    /**
     * Generate fake AdNet data (Proof of views and Proof of taps)
     */
    static generateAdNetData(): ThingSpeakData {
        // Based on your data pattern, field1 (Proof of views) has various ranges
        const proofOfViews = faker.helpers.weightedArrayElement([
            { weight: 0.3, value: faker.number.int({ min: 0, max: 10 }) },
            { weight: 0.4, value: faker.number.int({ min: 11, max: 30 }) },
            { weight: 0.2, value: faker.number.int({ min: 31, max: 60 }) },
            { weight: 0.1, value: faker.number.int({ min: 61, max: 100 }) },
        ]);

        // field2 (Proof of taps) - usually lower than views
        const proofOfTaps = faker.number.int({ min: 0, max: Math.min(20, proofOfViews) });

        const data = {
            field1: proofOfViews,  // Proof of views
            field2: proofOfTaps,   // Proof of taps
        };

        // Validate generated data before returning
        return ThingSpeakDataSchema.parse(data);
    }

    /**
     * Generate multiple fake entries
     */
    static generateMultipleEntries(count: number): ThingSpeakData[] {
        const entries: ThingSpeakData[] = [];

        for (let i = 0; i < count; i++) {
            try {
                entries.push(this.generateAdNetData());
            } catch (error) {
                if (error instanceof z.ZodError) {
                    console.error(`Validation error for entry ${i + 1}:`, error.errors);
                    throw new Error(`Failed to generate valid fake data: ${error.errors.map(e => e.message).join(', ')}`);
                }
                throw error;
            }
        }

        return entries;
    }
}

// Usage example
async function main() {
    try {
        // Replace with your actual ThingSpeak Write API Key
        const API_KEY = 'C9A45GHUP7N569HJ';

        const sender = new ThingSpeakSender({
            apiKey: API_KEY,
            channelId: 2890626, // Your AdNet Module View test channel (auto-set as default)
        });

        // Read current channel data
        console.log('Reading current channel data...');
        const channelData = await sender.readChannelData(10); // Get last 10 entries
        console.log('Channel Info:', channelData.channel.name);
        console.log('Latest entries:', channelData.feeds.slice(-3));

        // Get channel statistics
        console.log('\nGetting channel statistics...');
        const stats = await sender.getChannelStats();
        console.log('Channel Stats:', stats);

        // Send single fake entry
        console.log('\nSending single fake entry...');
        const fakeData = FakeDataGenerator.generateAdNetData();
        console.log('Generated data:', fakeData);

        const result = await sender.sendDataViaParams(fakeData);
        console.log('Result:', result);

        // Read specific field data (Proof of views)
        console.log('\nReading Proof of Views data...');
        const viewsData = await sender.readFieldData(1, 5); // Last 5 view entries
        console.log('Recent views:', viewsData.feeds.map(f => ({
            views: f.field1,
            time: f.created_at
        })));

        // Send multiple entries (uncomment to use)
        /*
        console.log('\nSending multiple fake entries...');
        const fakeEntries = FakeDataGenerator.generateMultipleEntries(5);
        await sender.sendBulkData(fakeEntries, 16000); // 16 second delay
        */

    } catch (error) {
        if (error instanceof z.ZodError) {
            console.error('Validation error:', error.errors);
            console.error('Please check your configuration and data format');
        } else {
            console.error('Error in main:', error);
        }
    }
}

// Export for use in other modules
export {
    ThingSpeakSender,
    FakeDataGenerator,
    ThingSpeakData,
    ThingSpeakConfig,
    ThingSpeakResponse,
    ThingSpeakChannel,
    ThingSpeakChannelFeed,
    ThingSpeakDataSchema,
    ThingSpeakResponseSchema,
    ThingSpeakConfigSchema,
    ThingSpeakChannelSchema
};

// Run if this file is executed directly
if (require.main === module) {
    main();
}