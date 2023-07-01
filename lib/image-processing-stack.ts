import { Construct } from 'constructs';
import * as cdk from 'aws-cdk-lib';
import * as apigateway from "aws-cdk-lib/aws-apigateway";
import * as lambda from 'aws-cdk-lib/aws-lambda';
import * as path from 'path';
import * as awsLogs from 'aws-cdk-lib/aws-logs';

export class ImageProcessingStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    const handler = new lambda.Function(this, "Handler", {
      runtime: lambda.Runtime.PROVIDED_AL2,
      code: lambda.Code.fromAsset(path.join(__dirname, "..", "functions/image-processing/target/lambda/get_image")),
      handler: "dummy",
      logRetention: awsLogs.RetentionDays.FIVE_DAYS,
      memorySize: 128,
      tracing: lambda.Tracing.ACTIVE,
      timeout: cdk.Duration.minutes(3)
    });

    const restApi = new apigateway.RestApi(this, "Api", {
      restApiName: "ImageProcessing-Api",
      description: "ApiGateway for image processing handlers",
      binaryMediaTypes: ["*/*"]
    });

    const api = restApi.root.addResource("api");
    const imageEndpoint = api.addResource("image");
    imageEndpoint.addMethod("GET", new apigateway.LambdaIntegration(handler));
  }
}
