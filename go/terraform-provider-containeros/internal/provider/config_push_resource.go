package provider

import (
	"context"
	"fmt"
	"time"

	"github.com/hashicorp/terraform-plugin-framework/resource"
	"github.com/hashicorp/terraform-plugin-framework/resource/schema"
	"github.com/hashicorp/terraform-plugin-framework/types"
	"github.com/hashicorp/terraform-plugin-log/tflog"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"

	pb "terraform-provider-containeros/internal/rpc"
)

var _ resource.Resource = &ConfigPushResource{}
var _ pb.ConfigPullRequest = pb.ConfigPullRequest{}

func NewExampleResource() resource.Resource {
	return &ConfigPushResource{}
}

type ConfigPushResource struct{}

type ConfigPushResourceModel struct {
	Config types.String `tfsdk:"config"`
	Server types.String `tfsdk:"server"`
}

func (r *ConfigPushResource) Metadata(ctx context.Context, req resource.MetadataRequest, resp *resource.MetadataResponse) {
	resp.TypeName = req.ProviderTypeName + "_config_push"
}

func (r *ConfigPushResource) Schema(ctx context.Context, req resource.SchemaRequest, resp *resource.SchemaResponse) {
	resp.Schema = schema.Schema{
		MarkdownDescription: "Push config to a server",

		Attributes: map[string]schema.Attribute{
			"config": schema.StringAttribute{
				MarkdownDescription: "The YAML config",
				Required:            true,
				Sensitive:           true,
			},
			"server": schema.StringAttribute{
				MarkdownDescription: "The target server",
				Required:            true,
			},
		},
	}
}

func (r *ConfigPushResource) Configure(ctx context.Context, req resource.ConfigureRequest, resp *resource.ConfigureResponse) {
	if req.ProviderData == nil {
		return
	}
}

func (r *ConfigPushResource) Create(ctx context.Context, req resource.CreateRequest, resp *resource.CreateResponse) {
	var data ConfigPushResourceModel

	resp.Diagnostics.Append(req.Plan.Get(ctx, &data)...)
	if resp.Diagnostics.HasError() {
		return
	}

	addr := data.Server.ValueString()
	conn, err := grpc.NewClient(addr, grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		resp.Diagnostics.AddError("Client Error", fmt.Sprintf("Unable to push config, got error: %s", err))
	}
	defer conn.Close()

	c := pb.NewApiServiceClient(conn)
	ctx, cancel := context.WithTimeout(context.Background(), time.Second)
	defer cancel()

	_, err = c.ConfigPushStr(ctx, &pb.ConfigPushStrRequest{Yaml: data.Config.ValueString()})
	if err != nil {
		resp.Diagnostics.AddError("Client Error", fmt.Sprintf("Unable to push config, got error: %s", err))
	}

	tflog.Trace(ctx, "pushed config")
	resp.Diagnostics.Append(resp.State.Set(ctx, &data)...)
}

func (r *ConfigPushResource) Read(ctx context.Context, req resource.ReadRequest, resp *resource.ReadResponse) {
	var data ConfigPushResourceModel
	resp.Diagnostics.Append(req.State.Get(ctx, &data)...)
	if resp.Diagnostics.HasError() {
		return
	}

	resp.Diagnostics.Append(resp.State.Set(ctx, &data)...)
}

func (r *ConfigPushResource) Update(ctx context.Context, req resource.UpdateRequest, resp *resource.UpdateResponse) {
	var data ConfigPushResourceModel

	resp.Diagnostics.Append(req.Plan.Get(ctx, &data)...)
	if resp.Diagnostics.HasError() {
		return
	}

	addr := data.Server.ValueString()
	conn, err := grpc.NewClient(addr, grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		resp.Diagnostics.AddError("Client Error", fmt.Sprintf("Unable to push config, got error: %s", err))
	}
	defer conn.Close()

	c := pb.NewApiServiceClient(conn)
	ctx, cancel := context.WithTimeout(context.Background(), time.Second)
	defer cancel()

	_, err = c.ConfigPushStr(ctx, &pb.ConfigPushStrRequest{Yaml: data.Config.ValueString()})
	if err != nil {
		resp.Diagnostics.AddError("Client Error", fmt.Sprintf("Unable to push config, got error: %s", err))
	}

	tflog.Trace(ctx, "pushed config")
	resp.Diagnostics.Append(resp.State.Set(ctx, &data)...)
}

func (r *ConfigPushResource) Delete(ctx context.Context, req resource.DeleteRequest, resp *resource.DeleteResponse) {
	var data ConfigPushResourceModel
	resp.Diagnostics.Append(req.State.Get(ctx, &data)...)
	if resp.Diagnostics.HasError() {
		return
	}
}
