(DONE) Upgrade to latest protos.

(DONE) Everything that says 'choice' in stream should be 'output'.

(DONE) Replace TokenContext and CompletionContext with OutputContext
    OutputContext {
        pub total_outputs: usize,
        pub output_index: usize,
        pub reasoning_status: PhaseStatus,
        pub content_status: PhaseStatus,
    }

Add following to chat::stream::Consumer
* order functions by execution order
    on_chunk(&chunk)  -- on all chunks

    on_reason_token(&out_ctx, &tkn), -- per delta "of first half"
    on_reasoning_complete(&out_ctx), -- "first half" done

    on_content_token(&out_ctx, &tkn), -- per delta "of second half"
    on_content_complete(&out_ctx), -- "second half" done

    on_inline_citations(&out_ctx, &citations) -- idk
    on_tool_calls(&out_ctx, &calls) -- last delta per output

    on_usage(&usage) -- on last chunk
    on_citations(&citations) -- on last chunk
