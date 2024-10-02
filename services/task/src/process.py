import base64
import os
import tempfile
import threading
from paddleocr import PaddleOCR, PPStructure
from pathlib import Path

from src.configs.llm_config import LLM__BASE_URL
from src.converters import crop_image
from src.llm import process_table
from src.models.segment_model import BaseSegment, Segment, SegmentType
from src.ocr import ppocr, ppstructure_table
from src.s3 import upload_file_to_s3

def adjust_base_segments(segments: list[BaseSegment], offset: float = 5.0, density: int = 300, pdla_density: int = 72):
    scale_factor = density / pdla_density
    for segment in segments:
        segment.width *= scale_factor
        segment.height *= scale_factor
        segment.left *= scale_factor
        segment.top *= scale_factor

        segment.page_height *= scale_factor
        segment.page_width *= scale_factor

        segment.width += offset * 2
        segment.height += offset * 2
        segment.left -= offset
        segment.top -= offset

        segment.left = max(0, segment.left)
        segment.top = max(0, segment.top)
        segment.width = min(segment.width, segment.page_width)
        segment.height = min(segment.height, segment.page_height)


def process_segment_ocr(
    segment: Segment,
    segment_temp_file: Path,
    ocr: PaddleOCR,
    table_engine: PPStructure,
    ocr_lock: threading.Lock,
    table_engine_lock: threading.Lock
):
    if segment.segment_type == SegmentType.Table:
        if LLM__BASE_URL:
            segment.html = process_table(segment_temp_file)
        else:
            with table_engine_lock:
                table_ocr_results = ppstructure_table(
                    table_engine, segment_temp_file)
                segment.ocr = table_ocr_results.results
                segment.html = table_ocr_results.html
    elif segment.segment_type == SegmentType.Picture:
        with ocr_lock:
            ocr_results = ppocr(ocr, segment_temp_file)
            segment.ocr = ocr_results.results
    else:
        with ocr_lock:
            ocr_results = ppocr(ocr, segment_temp_file)
        segment.ocr = ocr_results.results


def process_segment(
    segment: Segment,
    image_folder_location: str,
    page_image_file_paths: dict[int, Path],
    segment_image_extension: str,
    segment_image_quality: int,
    segment_image_resize: str,
    ocr_strategy: str,
    ocr: PaddleOCR,
    table_engine: PPStructure,
    ocr_lock: threading.Lock,
    table_engine_lock: threading.Lock
) -> Segment:
    try:
        ocr_needed = ocr_strategy == "All" or (
            ocr_strategy != "Off" and (
                segment.segment_type in [SegmentType.Table, SegmentType.Picture] or
                (ocr_strategy == "Auto" and not segment.content)
            )
        )

        if ocr_needed:
            base64_image = crop_image(
                page_image_file_paths[segment.page_number],
                segment.bbox,
                segment_image_extension,
                segment_image_quality,
                segment_image_resize
            )

            image_s3_path = f"{image_folder_location}/{segment.segment_id}.{segment_image_extension}"
            temp_image_file = tempfile.NamedTemporaryFile(
                suffix=f".{segment_image_extension}", delete=False)
            try:
                temp_image_file.write(base64.b64decode(base64_image))
                temp_image_file.close()
                upload_file_to_s3(
                    temp_image_file.name,
                    image_s3_path,
                )
                segment.image = image_s3_path
         
                process_segment_ocr(
                    segment,
                    Path(temp_image_file.name),
                    ocr,
                    table_engine,
                    ocr_lock,
                    table_engine_lock
                )
            finally:
                os.remove(temp_image_file.name)
    except Exception as e:
        print(
            f"Error processing segment {segment.segment_type} on page {segment.page_number}: {e}")
    finally:
        segment.finalize()
    return segment
