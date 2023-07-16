import * as cdk from 'aws-cdk-lib';
import * as apigateway from "aws-cdk-lib/aws-apigateway";

export class RestApiKey {
    public constructor(api: cdk.aws_apigateway.RestApi) {
        const usagePlan = api.addUsagePlan("UsagePlan", {
            apiStages: [{
                stage: api.deploymentStage,
            }],
            quota: {
                limit: 1000,
                period: apigateway.Period.DAY
            },
            throttle: {
                burstLimit: 10,
                rateLimit: 5
            },
        });

        const apiKey = api.addApiKey(`DevApiKey`, {
            apiKeyName: `ImageHandlerApiKey`,
            value: process.env.API_KEY
        });

        usagePlan.addApiKey(apiKey);
    }
}