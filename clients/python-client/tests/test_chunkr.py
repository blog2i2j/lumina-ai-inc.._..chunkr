import pytest
from pathlib import Path
from PIL import Image
import asyncio
import base64

from chunkr_ai import Chunkr
from chunkr_ai.models import (
    Configuration,
    GenerationStrategy,
    GenerationConfig,
    OcrStrategy,
    Pipeline,
    SegmentationStrategy,
    SegmentProcessing,
    ChunkProcessing,
    TaskResponse,
)

@pytest.fixture
def sample_path():
    return Path("tests/files/test.pdf")

@pytest.fixture
def sample_image():
    return Image.open("tests/files/test.jpg")

@pytest.fixture
def sample_url():
    return "https://chunkr-web.s3.us-east-1.amazonaws.com/landing_page/input/science.pdf"

@pytest.fixture
def client():
    client = Chunkr()
    yield client

@pytest.mark.asyncio
async def test_send_file_path(client, sample_path):
    response = await client.upload(sample_path)
    assert response.task_id is not None
    assert response.status == "Succeeded"
    assert response.output is not None

@pytest.mark.asyncio
async def test_send_file_url(client, sample_url):
    response = await client.upload(sample_url)
    assert response.task_id is not None
    assert response.status == "Succeeded"
    assert response.output is not None

@pytest.mark.asyncio
async def test_send_file_path_str(client, sample_path):
    response = await client.upload(str(sample_path))
    assert response.task_id is not None
    assert response.status == "Succeeded"
    assert response.output is not None

@pytest.mark.asyncio
async def test_send_opened_file(client, sample_path):
    with open(sample_path, "rb") as f:
        response = await client.upload(f)
    assert response.task_id is not None
    assert response.status == "Succeeded"
    assert response.output is not None

@pytest.mark.asyncio
async def test_send_pil_image(client, sample_image):
    response = await client.upload(sample_image)
    assert response.task_id is not None
    assert response.status == "Succeeded"
    assert response.output is not None
    assert response.output is not None

@pytest.mark.asyncio
async def test_ocr_auto(client, sample_path):
    response = await client.upload(sample_path, Configuration(ocr_strategy=OcrStrategy.AUTO))
    assert response.task_id is not None
    assert response.status == "Succeeded"
    assert response.output is not None

@pytest.mark.asyncio
async def test_expires_in(client, sample_path):
    response = await client.upload(sample_path, Configuration(expires_in=10))
    assert response.task_id is not None
    assert response.status == "Succeeded"
    assert response.output is not None
    await asyncio.sleep(11)
    try:
        await client.get_task(response.task_id)
        assert False
    except Exception as e:
        print(e)
        assert True

@pytest.mark.asyncio
async def test_chunk_processing(client, sample_path):
    response = await client.upload(
        sample_path,
        Configuration(chunk_processing=ChunkProcessing(target_length=1024)),
    )
    assert response.task_id is not None
    assert response.status == "Succeeded"
    assert response.output is not None

@pytest.mark.asyncio
async def test_segmentation_strategy_page(client, sample_path):
    response = await client.upload(
        sample_path, Configuration(segmentation_strategy=SegmentationStrategy.PAGE)
    )
    assert response.task_id is not None
    assert response.status == "Succeeded"
    assert response.output is not None

@pytest.mark.asyncio
async def test_page_llm_html(client, sample_path):
    response = await client.upload(
        sample_path,
        Configuration(
            segmentation_strategy=SegmentationStrategy.PAGE,
            segment_processing=SegmentProcessing(
                page=GenerationConfig(html=GenerationStrategy.LLM)
            ),
        ),
    )
    assert response.task_id is not None
    assert response.status == "Succeeded"
    assert response.output is not None

@pytest.mark.asyncio
async def test_page_llm(client, sample_path):
    configuration = Configuration(
        segmentation_strategy=SegmentationStrategy.PAGE,
        segment_processing=SegmentProcessing(
            page=GenerationConfig(
                html=GenerationStrategy.LLM, markdown=GenerationStrategy.LLM
            )
        ),
    )
    response = await client.upload(sample_path, configuration)
    assert response.task_id is not None
    assert response.status == "Succeeded"
    assert response.output is not None

@pytest.mark.asyncio
async def test_delete_task(client, sample_path):
    response = await client.upload(sample_path)
    await client.delete_task(response.task_id)
    with pytest.raises(Exception):
        await client.get_task(response.task_id)

@pytest.mark.asyncio
async def test_delete_task_direct(client, sample_path):
    task = await client.upload(sample_path)
    assert isinstance(task, TaskResponse)
    assert task.task_id is not None
    assert task.status == "Succeeded"
    assert task.output is not None
    await client.delete_task(task.task_id)
    with pytest.raises(Exception):
        await client.get_task(task.task_id)

@pytest.mark.asyncio
async def test_cancel_task(client, sample_path):
    response = await client.create_task(sample_path)
    assert response.status == "Starting"
    await client.cancel_task(response.task_id)
    assert (await client.get_task(response.task_id)).status == "Cancelled"

@pytest.mark.asyncio
async def test_cancel_task_direct(client, sample_path):
    task = await client.create_task(sample_path)
    assert isinstance(task, TaskResponse)
    assert task.status == "Starting"
    await task.cancel()
    assert task.status == "Cancelled"

@pytest.mark.asyncio
async def test_update_task(client, sample_path):
    original_config = Configuration(
        segmentation_strategy=SegmentationStrategy.LAYOUT_ANALYSIS,
    )
    new_config = Configuration(
        segmentation_strategy=SegmentationStrategy.PAGE,
    )
    task = await client.upload(sample_path, original_config)
    task = await client.update(task.task_id, new_config)
    assert task.status == "Succeeded"
    assert task.output is not None
    assert task.configuration.segmentation_strategy == SegmentationStrategy.PAGE

@pytest.mark.asyncio
async def test_update_task_direct(client, sample_path):
    original_config = Configuration(
        segmentation_strategy=SegmentationStrategy.LAYOUT_ANALYSIS,
    )
    new_config = Configuration(
        segmentation_strategy=SegmentationStrategy.PAGE,
    )
    task = await client.upload(sample_path, original_config)
    task = await task.update(new_config)
    assert task.status == "Succeeded"
    assert task.output is not None
    assert task.configuration.segmentation_strategy == SegmentationStrategy.PAGE

@pytest.mark.asyncio
async def test_pipeline_type_azure(client, sample_path):
    response = await client.upload(sample_path, Configuration(pipeline=Pipeline.AZURE))
    assert response.task_id is not None
    assert response.status == "Succeeded"
    assert response.output is not None
    
@pytest.mark.asyncio
async def test_pipeline_type_azure(client, sample_path):
    response = await client.upload(sample_path, Configuration(pipeline=Pipeline.CHUNKR))
    assert response.task_id is not None
    assert response.status == "Succeeded"
    assert response.output is not None

@pytest.mark.asyncio
async def test_client_lifecycle(client, sample_path):
    response1 = await client.upload(sample_path)
    await client.close()
    response2 = await client.upload(sample_path)
    assert response1.task_id is not None
    assert response2.task_id is not None

@pytest.mark.asyncio
async def test_task_operations_after_client_close(client, sample_path):
    task = await client.create_task(sample_path)
    await client.close()
    result = await task.poll()
    assert result.status == "Succeeded"

@pytest.mark.asyncio
async def test_send_base64_file(client, sample_path):
    # Read file and convert to base64
    with open(sample_path, "rb") as f:
        base64_content = base64.b64encode(f.read()).decode('utf-8')
    response = await client.upload(base64_content)
    assert response.task_id is not None
    assert response.status == "Succeeded"
    assert response.output is not None

@pytest.mark.asyncio
async def test_send_base64_file_with_filename(client, sample_path):
    # Read file and convert to base64
    with open(sample_path, "rb") as f:
        base64_content = base64.b64encode(f.read()).decode('utf-8')
    
    response = await client.upload(base64_content, filename="test.pdf")
    assert response.task_id is not None
    assert response.status == "Succeeded"
    assert response.output is not None
    
@pytest.mark.asyncio
async def test_output_files_no_dir(client, sample_path, tmp_path):
    task = await client.upload(sample_path)
    
    html_file = tmp_path / "output.html"
    md_file = tmp_path / "output.md"
    content_file = tmp_path / "output.txt"
    json_file = tmp_path / "output.json"
    
    task.html(html_file)
    task.markdown(md_file)
    task.content(content_file)
    task.json(json_file)
    
    assert html_file.exists()
    assert md_file.exists()
    assert content_file.exists()
    assert json_file.exists()

@pytest.mark.asyncio
async def test_output_files_with_dirs(client, sample_path, tmp_path):
    task = await client.upload(sample_path)
    
    nested_dir = tmp_path / "nested" / "output" / "dir"
    html_file = nested_dir / "output.html"
    md_file = nested_dir / "output.md"
    content_file = nested_dir / "output.txt"
    json_file = nested_dir / "output.json"
    
    task.html(html_file)
    task.markdown(md_file)
    task.content(content_file)
    task.json(json_file)

    assert html_file.exists()
    assert md_file.exists()
    assert content_file.exists()
    assert json_file.exists()